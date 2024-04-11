mod balance;
mod busses;
mod claim;
mod cu_limits;
mod mine;
mod register;
mod rewards;
mod send_and_confirm;
mod treasury;
mod utils;

use std::io::{self, BufRead};
use std::{fs::File, path::Path, sync::Arc};

use clap::{command, Parser, Subcommand};
use solana_sdk::signature::{read_keypair_file, Keypair};

struct Miner {
    pub keypair_filepath: Option<String>,
    pub priority_fee: u64,
    pub cluster: String,
    pub jito_fee: u64,
    pub jito_enable: bool,
    pub jito_client: String,
}

#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    #[arg(
    long,
    value_name = "JitoTips Fee",
    help = "10000=0.00001SOL",
    default_value = "10000"
    )]
    jito_fee: u64,

    #[arg(
    long,
    value_name = "enable JitoTips",
    help = "enable JitoTips?",
    default_value = "false"
    )]
    jito_enable: bool,
    #[arg(
        long,
        value_name = "NETWORK_URL",
        help = "Network address of your RPC provider",
        global = true
    )]
    rpc: Option<String>,
    #[arg(
        long,
        value_name = "JITO_URL",
        help = "Network address of your JITO RPC provider",
        global = true
    )]
    jito_client: Option<String>,

    #[clap(
        global = true,
        short = 'C',
        long = "config",
        id = "PATH",
        help = "Filepath to config file."
    )]
    pub config_file: Option<String>,

    #[arg(
        long,
        value_name = "KEYPAIR_FILEPATH",
        help = "Filepath to keypair to use",
        global = true
    )]
    keypair: Option<String>,

    #[arg(
        long,
        value_name = "MICROLAMPORTS",
        help = "Number of microlamports to pay as priority fee per transaction",
        default_value = "0",
        global = true
    )]
    priority_fee: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Fetch the Ore balance of an account")]
    Balance(BalanceArgs),

    #[command(about = "Fetch the distributable rewards of the busses")]
    Busses(BussesArgs),

    #[command(about = "Mine Ore using local compute")]
    Mine(MineArgs),

    #[command(about = "Claim available mining rewards")]
    Claim(ClaimArgs),

    #[command(about = "Fetch your balance of unclaimed mining rewards")]
    Rewards(RewardsArgs),

    #[command(about = "Fetch the treasury account and balance")]
    Treasury(TreasuryArgs),
}

#[derive(Parser, Debug)]
struct BalanceArgs {
    #[arg(
        // long,
        value_name = "ADDRESS",
        help = "The address of the account to fetch the balance of"
    )]
    pub address: Option<String>,
}

#[derive(Parser, Debug)]
struct BussesArgs {}

#[derive(Parser, Debug)]
struct RewardsArgs {
    #[arg(
        // long,
        value_name = "ADDRESS",
        help = "The address of the account to fetch the rewards balance of"
    )]
    pub address: Option<String>,
}

#[derive(Parser, Debug)]
struct MineArgs {
    #[arg(
        long,
        short,
        value_name = "THREAD_COUNT",
        help = "The number of threads to dedicate to mining",
        default_value = "1"
    )]
    threads: u64,
}

#[derive(Parser, Debug)]
struct TreasuryArgs {}

#[derive(Parser, Debug)]
struct ClaimArgs {
    #[arg(
        // long,
        value_name = "AMOUNT",
        help = "The amount of rewards to claim. Defaults to max."
    )]
    amount: Option<f64>,

    #[arg(
        // long,
        value_name = "TOKEN_ACCOUNT_ADDRESS",
        help = "Token account to receive mining rewards."
    )]
    beneficiary: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Load the config file from custom path, the default path, or use default config values
    let cli_config = if let Some(config_file) = &args.config_file {
        solana_cli_config::Config::load(config_file).unwrap_or_else(|_| {
            eprintln!("error: Could not find config file `{}`", config_file);
            std::process::exit(1);
        })
    } else if let Some(config_file) = &*solana_cli_config::CONFIG_FILE {
        solana_cli_config::Config::load(config_file).unwrap_or_default()
    } else {
        solana_cli_config::Config::default()
    };

    // Initialize miner.
    let cluster = args.rpc.unwrap_or(cli_config.json_rpc_url.clone());
    let jito_client = args.jito_client.unwrap_or(cli_config.json_rpc_url.clone());
    let keypair_paths = read_lines("wallets.txt");



    let mut miner_handles = Vec::new();

    if let Ok(keypair_path) = keypair_paths {
        println!("{}",keypair_path[0].clone());
        let miner = Miner::new(
            cluster.clone(),
            jito_client.clone(),
            args.priority_fee,
            Some(keypair_path[0].clone().to_string()),
            args.jito_fee,
            args.jito_enable,
        );

        // Spawn a new asynchronous task for each miner to start mining concurrently.
        let handle = tokio::task::spawn_blocking(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                miner.mine(8).await;
            });
        });
        
        miner_handles.push(handle);
    }
}

impl Miner {
    pub fn new(cluster: String, jito_client: String, priority_fee: u64, keypair_filepath: Option<String>, jito_fee: u64, jito_enable: bool) -> Self {
        Self {
            keypair_filepath,
            priority_fee,
            cluster,
            jito_client,
            jito_fee,
            jito_enable,
        }
    }

    pub fn signer(&self) -> Keypair {
        match self.keypair_filepath.clone() {
            Some(filepath) => read_keypair_file(filepath).unwrap(),
            None => panic!("No keypair provided"),
        }
    }
}

// Function to read lines from a file and return them as a Vec<String>.
fn read_lines<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    io::BufReader::new(file).lines().collect()
}