use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use cdk::mint_url::MintUrl;
use cdk::nuts::{CurrencyUnit, MintQuoteState, PaymentMethod};
use cdk::wallet::{ReceiveOptions, SendMemo, SendOptions, Wallet, WalletBuilder};
use cdk::Amount;
use cdk_supabase::{SupabaseAuthResponse, SupabaseWalletDatabase};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
enum AppError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Cdk(#[from] cdk::Error),
    #[error(transparent)]
    Database(#[from] cdk::cdk_database::Error),
    #[error(transparent)]
    Supabase(#[from] cdk_supabase::Error),
}

type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Parser)]
#[command(
    name = "supabase-wallet",
    about = "CDK wallet backed by Supabase storage"
)]
struct Cli {
    /// Supabase project URL, e.g. https://your-project.supabase.co/
    #[arg(long, global = true, env = "CDK_SUPABASE_URL")]
    supabase_url: Option<Url>,

    /// Supabase anon or publishable API key
    #[arg(long, global = true, env = "CDK_SUPABASE_ANON_KEY")]
    anon_key: Option<String>,

    /// Directory used for local-only wallet state such as the seed and saved session
    #[arg(
        long,
        global = true,
        env = "CDK_SUPABASE_STATE_DIR",
        default_value = ".local/supabase-wallet/state"
    )]
    state_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the SQL schema needed to bootstrap Supabase manually
    Schema,
    /// Check that the Supabase database schema is compatible with this version
    CheckSchema,
    /// Create a Supabase Auth user and save the returned session if available
    Signup(AuthArgs),
    /// Sign in with Supabase Auth and save the session locally
    Signin(AuthArgs),
    /// Remove the locally saved Supabase session
    ClearSession,
    /// Show balances, pending quotes, and recent transactions
    Status(WalletArgs),
    /// Show wallet balance
    Balance(WalletArgs),
    /// Request a mint quote (Lightning invoice) and wait for payment, then receive proofs
    Mint(MintArgs),
    /// Pay a Lightning invoice by melting tokens
    Melt(MeltArgs),
    /// Send tokens (outputs a Cashu token string)
    Send(SendArgs),
    /// Receive a Cashu token string
    Receive(ReceiveArgs),
}

// ---------------------------------------------------------------------------
// Shared wallet args — used by all commands that build a Wallet
// ---------------------------------------------------------------------------

#[derive(Debug, Args, Clone)]
struct WalletArgs {
    /// Cashu mint URL
    #[arg(long, env = "CDK_SUPABASE_MINT_URL")]
    mint_url: String,

    /// Wallet unit, e.g. sat
    #[arg(long, env = "CDK_SUPABASE_UNIT", default_value = "sat")]
    unit: String,

    /// Password used to encrypt wallet secrets in Supabase
    #[arg(long, env = "CDK_SUPABASE_ENCRYPTION_PASSWORD")]
    encryption_password: String,

    #[command(flatten)]
    credentials: CredentialsArgs,
}

// ---------------------------------------------------------------------------
// Per-command args
// ---------------------------------------------------------------------------


#[derive(Debug, Args)]
struct AuthArgs {
    #[command(flatten)]
    credentials: CredentialsArgs,
}

#[derive(Debug, Args)]
struct MintArgs {
    #[command(flatten)]
    wallet: WalletArgs,

    /// Amount to mint in the wallet's unit
    #[arg(long)]
    amount: u64,

    /// How long to wait for the invoice to be paid (seconds)
    #[arg(long, default_value_t = 300)]
    timeout: u64,

    /// Number of recent transactions to display after minting
    #[arg(long, default_value_t = 5)]
    recent_transactions: usize,
}

#[derive(Debug, Args)]
struct MeltArgs {
    #[command(flatten)]
    wallet: WalletArgs,

    /// BOLT11 invoice to pay
    #[arg(long)]
    invoice: String,

    /// Number of recent transactions to display after melting
    #[arg(long, default_value_t = 5)]
    recent_transactions: usize,
}

#[derive(Debug, Args)]
struct SendArgs {
    #[command(flatten)]
    wallet: WalletArgs,

    /// Amount to send in the wallet's unit
    #[arg(long)]
    amount: u64,

    /// Optional memo to attach to the token
    #[arg(long)]
    memo: Option<String>,
}

#[derive(Debug, Args)]
struct ReceiveArgs {
    #[command(flatten)]
    wallet: WalletArgs,

    /// Cashu token string (cashuA… / cashuB…)
    #[arg(long)]
    token: String,
}

// ---------------------------------------------------------------------------
// Credentials
// ---------------------------------------------------------------------------

#[derive(Debug, Args, Clone)]
struct CredentialsArgs {
    /// Supabase Auth email
    #[arg(long, env = "CDK_SUPABASE_EMAIL")]
    email: Option<String>,

    /// Supabase Auth password
    #[arg(long, env = "CDK_SUPABASE_PASSWORD")]
    password: Option<String>,
}

#[derive(Debug, Clone)]
struct Credentials {
    email: String,
    password: String,
}

impl CredentialsArgs {
    fn resolve(&self) -> Result<Option<Credentials>> {
        match (&self.email, &self.password) {
            (Some(email), Some(password)) => Ok(Some(Credentials {
                email: email.clone(),
                password: password.clone(),
            })),
            (None, None) => Ok(None),
            _ => Err(AppError::Message(
                "Provide both --email and --password, or neither.".to_string(),
            )),
        }
    }

    fn require(&self) -> Result<Credentials> {
        self.resolve()?.ok_or_else(|| {
            AppError::Message(
                "Missing Supabase Auth credentials. Use --email/--password or the matching CDK_SUPABASE_* environment variables.".to_string(),
            )
        })
    }
}

// ---------------------------------------------------------------------------
// Local state paths
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct StatePaths {
    root: PathBuf,
    seed: PathBuf,
    session: PathBuf,
}

impl StatePaths {
    fn new(root: PathBuf) -> Self {
        Self {
            seed: root.join("seed.json"),
            session: root.join("session.json"),
            root,
        }
    }

    fn ensure_root(&self) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedSeed {
    seed: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SavedSession {
    email: String,
    user_id: Option<String>,
    access_token: String,
    refresh_token: Option<String>,
}

impl SavedSession {
    fn from_auth_response(email: String, response: &SupabaseAuthResponse) -> Option<Self> {
        if response.access_token.is_empty() {
            return None;
        }

        let user_id = response
            .user
            .get("id")
            .and_then(|value| value.as_str())
            .map(str::to_owned);

        Some(Self {
            email,
            user_id,
            access_token: response.access_token.clone(),
            refresh_token: response.refresh_token.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let state_paths = StatePaths::new(cli.state_dir.clone());

    match &cli.command {
        Command::Schema => {
            println!("{}", SupabaseWalletDatabase::get_schema_sql());
        }
        Command::CheckSchema => check_schema(&cli).await?,
        Command::Signup(args) => signup(&cli, &state_paths, args).await?,
        Command::Signin(args) => signin(&cli, &state_paths, args).await?,
        Command::ClearSession => clear_session(&state_paths)?,
        Command::Status(args) => status(&cli, &state_paths, args).await?,
        Command::Balance(args) => balance(&cli, &state_paths, args).await?,
        Command::Mint(args) => mint(&cli, &state_paths, args).await?,
        Command::Melt(args) => melt(&cli, &state_paths, args).await?,
        Command::Send(args) => send(&cli, &state_paths, args).await?,
        Command::Receive(args) => receive(&cli, &state_paths, args).await?,
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Auth / setup commands
// ---------------------------------------------------------------------------

async fn check_schema(cli: &Cli) -> Result<()> {
    let database = create_database(cli).await?;
    database.check_schema_compatibility().await?;
    println!("Schema is compatible.");
    Ok(())
}

async fn signup(cli: &Cli, state_paths: &StatePaths, args: &AuthArgs) -> Result<()> {
    let credentials = args.credentials.require()?;
    let database = create_database(cli).await?;
    let response = database
        .signup(&credentials.email, &credentials.password)
        .await?;

    if let Some(session) = SavedSession::from_auth_response(credentials.email.clone(), &response) {
        save_session(state_paths, &session)?;
        println!(
            "Signed up {} and saved a local session to {}.",
            session.email,
            state_paths.session.display()
        );
    } else {
        println!(
            "Signed up {}. No access token was returned, so no session was saved. Your Supabase project may require email confirmation before sign-in.",
            credentials.email
        );
    }

    Ok(())
}

async fn signin(cli: &Cli, state_paths: &StatePaths, args: &AuthArgs) -> Result<()> {
    let credentials = args.credentials.require()?;
    let database = create_database(cli).await?;
    let response = database
        .signin(&credentials.email, &credentials.password)
        .await?;

    let session = SavedSession::from_auth_response(credentials.email.clone(), &response)
        .ok_or_else(|| AppError::Message("Supabase did not return an access token.".to_string()))?;

    save_session(state_paths, &session)?;
    println!(
        "Signed in {} and saved the local session to {}.",
        session.email,
        state_paths.session.display()
    );

    Ok(())
}

fn clear_session(state_paths: &StatePaths) -> Result<()> {
    if state_paths.session.exists() {
        fs::remove_file(&state_paths.session)?;
        println!(
            "Removed saved session at {}.",
            state_paths.session.display()
        );
    } else {
        println!(
            "No saved session found at {}.",
            state_paths.session.display()
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Wallet commands
// ---------------------------------------------------------------------------

async fn balance(cli: &Cli, state_paths: &StatePaths, args: &WalletArgs) -> Result<()> {
    let (wallet, session) = build_wallet(cli, state_paths, args, "balance").await?;
    let unit = parse_currency_unit(&args.unit)?;

    let balance = wallet.total_balance().await?;
    let pending = wallet.total_pending_balance().await?;
    let reserved = wallet.total_reserved_balance().await?;

    println!("user:     {}", session.email);
    println!("balance:  {} {}", balance, unit);
    if pending > Amount::ZERO {
        println!("pending:  {} {}", pending, unit);
    }
    if reserved > Amount::ZERO {
        println!("reserved: {} {}", reserved, unit);
    }

    Ok(())
}

async fn status(cli: &Cli, state_paths: &StatePaths, args: &WalletArgs) -> Result<()> {
    let (wallet, session) = build_wallet(cli, state_paths, args, "status").await?;
    let unit = parse_currency_unit(&args.unit)?;
    let mint_url = parse_mint_url(&args.mint_url)?;

    let balance = wallet.total_balance().await?;
    let pending_balance = wallet.total_pending_balance().await?;
    let reserved_balance = wallet.total_reserved_balance().await?;
    let unspent_proofs = wallet.get_unspent_proofs().await?;
    let active_quotes = wallet.get_active_mint_quotes().await?;
    let unissued_quotes = wallet.get_unissued_mint_quotes().await?;
    let transactions = wallet.list_transactions(None).await?;

    println!("mint:              {}", mint_url);
    println!("unit:              {}", unit);
    println!(
        "user:              {}{}",
        session.email,
        session
            .user_id
            .as_ref()
            .map(|id| format!(" ({})", id))
            .unwrap_or_default()
    );
    println!("seed file:         {}", state_paths.seed.display());
    println!("balance:           {} {}", balance, unit);
    println!("pending balance:   {} {}", pending_balance, unit);
    println!("reserved balance:  {} {}", reserved_balance, unit);
    println!("unspent proofs:    {}", unspent_proofs.len());
    println!("active quotes:     {}", active_quotes.len());
    println!("unissued quotes:   {}", unissued_quotes.len());
    println!("transactions:      {}", transactions.len());

    let recent = 10;
    if !transactions.is_empty() {
        println!("\nrecent transactions (up to {}):", recent);
        for tx in transactions.iter().take(recent) {
            println!(
                "  {} {} {} fee={} id={} timestamp={}",
                tx.direction,
                tx.amount,
                tx.unit,
                tx.fee,
                tx.id(),
                tx.timestamp
            );
            if let Some(memo) = &tx.memo {
                println!("    memo: {}", memo);
            }
            if let Some(quote_id) = &tx.quote_id {
                println!("    quote_id: {}", quote_id);
            }
        }
    }

    Ok(())
}

async fn mint(cli: &Cli, state_paths: &StatePaths, args: &MintArgs) -> Result<()> {
    let (wallet, _session) = build_wallet(cli, state_paths, &args.wallet, "mint").await?;
    let unit = parse_currency_unit(&args.wallet.unit)?;
    let amount = Amount::from(args.amount);

    // Request a mint quote — the mint returns a Lightning invoice to pay
    let quote = wallet
        .mint_quote(PaymentMethod::BOLT11, Some(amount), None, None)
        .await?;

    println!("mint quote id: {}", quote.id);
    println!("invoice:       {}", quote.request);
    println!("amount:        {} {}", amount, unit);
    println!("expires at:    {}", quote.expiry);
    println!();
    println!("Pay the invoice above, then this command will continue automatically.");

    // Poll the mint until the invoice is paid, then collect proofs.
    // We use HTTP polling rather than the WebSocket stream so this works with
    // any mint regardless of WebSocket / NUT-17 support.
    let poll_interval = Duration::from_secs(3);
    let deadline = std::time::Instant::now() + Duration::from_secs(args.timeout);

    let proofs = loop {
        tokio::time::sleep(poll_interval).await;

        let updated = wallet.check_mint_quote_status(&quote.id).await?;

        match updated.state {
            MintQuoteState::Paid => {
                // Invoice paid — collect the proofs
                let proofs = wallet
                    .mint(&quote.id, Default::default(), None)
                    .await?;
                break proofs;
            }
            MintQuoteState::Issued => {
                return Err(AppError::Message(
                    "Quote already issued (tokens were already minted for this quote).".to_string(),
                ));
            }
            MintQuoteState::Unpaid => {
                if std::time::Instant::now() >= deadline {
                    return Err(AppError::Message(format!(
                        "Timed out after {}s waiting for invoice payment.",
                        args.timeout
                    )));
                }
                // still waiting — loop
            }
        }
    };

    let minted: Amount = proofs
        .iter()
        .map(|p| p.amount)
        .fold(Amount::ZERO, |acc, a| acc + a);

    println!();
    println!("minted:  {} {}", minted, unit);
    println!("balance: {} {}", wallet.total_balance().await?, unit);

    if args.recent_transactions > 0 {
        print_recent_transactions(&wallet, args.recent_transactions).await?;
    }

    Ok(())
}

async fn melt(cli: &Cli, state_paths: &StatePaths, args: &MeltArgs) -> Result<()> {
    let (wallet, _session) = build_wallet(cli, state_paths, &args.wallet, "melt").await?;
    let unit = parse_currency_unit(&args.wallet.unit)?;

    let quote = wallet
        .melt_quote(PaymentMethod::BOLT11, args.invoice.clone(), None, None)
        .await?;

    println!("melt quote id: {}", quote.id);
    println!("amount:        {} {}", quote.amount, unit);
    println!("fee reserve:   {} {}", quote.fee_reserve, unit);

    let prepared = wallet
        .prepare_melt(&quote.id, HashMap::new())
        .await?;

    println!(
        "total cost:    {} {} (amount + fee)",
        prepared.total_fee() + prepared.amount(),
        unit
    );

    let outcome = prepared.confirm_prefer_async().await?;

    match outcome {
        cdk::wallet::MeltOutcome::Paid(finalized) => {
            println!();
            println!("payment:  paid");
            println!("amount:   {} {}", finalized.amount(), unit);
            println!("fee paid: {} {}", finalized.fee_paid(), unit);
        }
        cdk::wallet::MeltOutcome::Pending(pending) => {
            println!("payment pending — waiting for confirmation…");
            let finalized = pending.await?;
            println!();
            println!("payment:  paid");
            println!("amount:   {} {}", finalized.amount(), unit);
            println!("fee paid: {} {}", finalized.fee_paid(), unit);
        }
    }

    println!("balance: {} {}", wallet.total_balance().await?, unit);

    if args.recent_transactions > 0 {
        print_recent_transactions(&wallet, args.recent_transactions).await?;
    }

    Ok(())
}

async fn send(cli: &Cli, state_paths: &StatePaths, args: &SendArgs) -> Result<()> {
    let (wallet, _session) = build_wallet(cli, state_paths, &args.wallet, "send").await?;
    let unit = parse_currency_unit(&args.wallet.unit)?;
    let amount = Amount::from(args.amount);

    let prepared = wallet
        .prepare_send(amount, SendOptions::default())
        .await?;

    let memo = args.memo.as_deref().map(SendMemo::for_token);
    let token = prepared.confirm(memo).await?;

    println!("{}", token);
    println!();
    println!("sent:    {} {}", amount, unit);
    println!("balance: {} {}", wallet.total_balance().await?, unit);

    Ok(())
}

async fn receive(cli: &Cli, state_paths: &StatePaths, args: &ReceiveArgs) -> Result<()> {
    let (wallet, _session) = build_wallet(cli, state_paths, &args.wallet, "receive").await?;
    let unit = parse_currency_unit(&args.wallet.unit)?;

    let received = wallet
        .receive(&args.token, ReceiveOptions::default())
        .await?;

    println!("received: {} {}", received, unit);
    println!("balance:  {} {}", wallet.total_balance().await?, unit);

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a fully-initialised wallet: authenticate, load/create seed, recover sagas.
async fn build_wallet(
    cli: &Cli,
    state_paths: &StatePaths,
    wallet_args: &WalletArgs,
    command_name: &str,
) -> Result<(Wallet, SavedSession)> {
    let (database, session) = authenticate_database(
        cli,
        state_paths,
        &wallet_args.encryption_password,
        &wallet_args.credentials,
        command_name,
    )
    .await?;

    let seed = load_or_create_seed(state_paths)?;
    let unit = parse_currency_unit(&wallet_args.unit)?;
    let mint_url = parse_mint_url(&wallet_args.mint_url)?;

    let wallet = WalletBuilder::new()
        .mint_url(mint_url)
        .unit(unit)
        .localstore(database)
        .seed(seed)
        .build()?;

    wallet.recover_incomplete_sagas().await?;

    Ok((wallet, session))
}

async fn print_recent_transactions(wallet: &Wallet, count: usize) -> Result<()> {
    let transactions = wallet.list_transactions(None).await?;
    if transactions.is_empty() {
        return Ok(());
    }
    println!();
    println!("recent transactions (up to {}):", count);
    for tx in transactions.iter().take(count) {
        println!(
            "  {} {} {} fee={} id={}",
            tx.direction, tx.amount, tx.unit, tx.fee, tx.id()
        );
        if let Some(memo) = &tx.memo {
            println!("    memo: {}", memo);
        }
    }
    Ok(())
}

async fn create_database(cli: &Cli) -> Result<Arc<SupabaseWalletDatabase>> {
    let supabase_url = require_supabase_project_url(
        cli.supabase_url.as_ref(),
        "--supabase-url or CDK_SUPABASE_URL",
    )?;
    let anon_key = require_string(cli.anon_key.as_ref(), "--anon-key or CDK_SUPABASE_ANON_KEY")?;

    Ok(Arc::new(
        SupabaseWalletDatabase::with_supabase_auth(supabase_url, anon_key).await?,
    ))
}

async fn apply_session(database: &Arc<SupabaseWalletDatabase>, session: &SavedSession) {
    database
        .set_jwt_token(Some(session.access_token.clone()))
        .await;
    database
        .set_refresh_token(session.refresh_token.clone())
        .await;
}

async fn authenticate_database(
    cli: &Cli,
    state_paths: &StatePaths,
    encryption_password: &str,
    credentials_args: &CredentialsArgs,
    command_name: &str,
) -> Result<(Arc<SupabaseWalletDatabase>, SavedSession)> {
    let database = create_database(cli).await?;
    database.set_encryption_password(encryption_password).await;

    let session = match credentials_args.resolve()? {
        Some(credentials) => {
            let response = database
                .signin(&credentials.email, &credentials.password)
                .await?;
            let session = SavedSession::from_auth_response(credentials.email.clone(), &response)
                .ok_or_else(|| {
                    AppError::Message(
                        "Supabase sign-in did not return an access token.".to_string(),
                    )
                })?;
            save_session(state_paths, &session)?;
            session
        }
        None => {
            let session = load_session(state_paths)?.ok_or_else(|| {
                AppError::Message(format!(
                    "No saved session found. Run `signin` first or pass --email and --password to `{command_name}`."
                ))
            })?;
            apply_session(&database, &session).await;
            session
        }
    };

    Ok((database, session))
}

fn load_or_create_seed(state_paths: &StatePaths) -> Result<[u8; 64]> {
    if state_paths.seed.exists() {
        let bytes = fs::read(&state_paths.seed)?;
        let saved_seed: SavedSeed = serde_json::from_slice(&bytes)?;

        if saved_seed.seed.len() != 64 {
            return Err(AppError::Message(format!(
                "Seed file {} does not contain 64 bytes.",
                state_paths.seed.display()
            )));
        }

        let mut seed = [0u8; 64];
        seed.copy_from_slice(&saved_seed.seed);
        return Ok(seed);
    }

    state_paths.ensure_root()?;
    let seed = rand::random::<[u8; 64]>();
    let saved_seed = SavedSeed {
        seed: seed.to_vec(),
    };
    fs::write(&state_paths.seed, serde_json::to_vec_pretty(&saved_seed)?)?;

    println!(
        "Created a new local wallet seed at {}.",
        state_paths.seed.display()
    );

    Ok(seed)
}

fn save_session(state_paths: &StatePaths, session: &SavedSession) -> Result<()> {
    state_paths.ensure_root()?;
    fs::write(&state_paths.session, serde_json::to_vec_pretty(session)?)?;
    Ok(())
}

fn load_session(state_paths: &StatePaths) -> Result<Option<SavedSession>> {
    if !state_paths.session.exists() {
        return Ok(None);
    }

    let bytes = fs::read(&state_paths.session)?;
    let session: SavedSession = serde_json::from_slice(&bytes)?;
    Ok(Some(session))
}

fn require_string(value: Option<&String>, flag_name: &str) -> Result<String> {
    value
        .cloned()
        .ok_or_else(|| AppError::Message(format!("Missing required value for {flag_name}.")))
}

fn require_url(value: Option<&Url>, flag_name: &str) -> Result<Url> {
    value
        .cloned()
        .ok_or_else(|| AppError::Message(format!("Missing required value for {flag_name}.")))
}

fn require_supabase_project_url(value: Option<&Url>, flag_name: &str) -> Result<Url> {
    let url = require_url(value, flag_name)?;

    match url.scheme() {
        "http" | "https" => Ok(url),
        scheme => Err(AppError::Message(format!(
            "Invalid Supabase project URL `{url}`. Use the HTTPS project URL like `https://<project-ref>.supabase.co`, not a `{scheme}` database connection string."
        ))),
    }
}

fn parse_currency_unit(value: &str) -> Result<CurrencyUnit> {
    CurrencyUnit::from_str(value).map_err(|error| {
        AppError::Message(format!(
            "Invalid unit `{value}`. Expected a valid CDK currency unit: {error}"
        ))
    })
}

fn parse_mint_url(value: &str) -> Result<MintUrl> {
    MintUrl::from_str(value)
        .map_err(|error| AppError::Message(format!("Invalid mint URL: {error}")))
}
