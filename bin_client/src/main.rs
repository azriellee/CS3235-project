// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This is the client program that covers the following tasks:
/// 1. File I/O. Read the config file and state files for initialization, dump the state files, etc.
/// 2. Read user input (using terminal UI) about transaction creation or quitting.
/// 3. Display the status and logs to the user (using terminal UI).
/// 4. IPC communication with the bin_nakamoto and the bin_wallet processes.

use seccompiler;
use seccompiler::{BpfProgram, BpfMap};

use tui::{backend::CrosstermBackend, Terminal};
use tui_textarea::{Input, Key};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::fs::File;
use std::io::{self, Write, BufReader, BufRead};
use std::process::{Command, Stdio};
use std::collections::BTreeMap;
use std::time::SystemTime;

use std::{thread, time::{Duration, Instant}};
use std::sync::{Mutex, Arc};
use serde::{Serialize, Deserialize};
use serde_json;

use std::fs;

mod app;

/// The enum type for the IPC messages (requests) from this client to the bin_nakamoto process.
/// It is the same as the `IPCMessageRequest` enum type in the bin_nakamoto process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReqNakamoto {
    Initialize(String, String, String),
    GetAddressBalance(String),
    PublishTx(String, String),
    RequestBlock(String),
    RequestNetStatus,
    RequestChainStatus,
    RequestMinerStatus,
    RequestTxPoolStatus,
    RequestStateSerialization,
    Quit,
}

/// The enum type for the IPC messages (responses) from the bin_nakamoto process to this client.
/// It is the same as the enum type in the bin_nakamoto process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageRespNakamoto {
    Initialized,
    PublishTxDone,
    AddressBalance(String, i64),
    BlockData(String),
    NetStatus(BTreeMap<String, String>),
    ChainStatus(BTreeMap<String, String>),
    MinerStatus(BTreeMap<String, String>),
    TxPoolStatus(BTreeMap<String, String>),
    StateSerialization(String, String),
    Quitting,
    Notify(String), 
}

/// The enum type for the IPC messages (requests) from this client to the bin_wallet process.
/// It is the same as the enum type in the bin_wallet process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReqWallet {
    Initialize(String),
    Quit,
    SignRequest(String),
    VerifyRequest(String, String),
    GetUserInfo,
}

/// The enum type for the IPC messages (responses) from the bin_wallet process to this client.
/// It is the same as the enum type in the bin_wallet process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageRespWallet {
    Initialized,
    Quitting,
    SignResponse(String, String),
    VerifyResponse(bool, String),
    UserInfo(String, String),
}

/// The enum type representing bot commands for controlling the client automatically.
/// The commands are read from a file or a named pipe and then executed by the client.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum BotCommand {
    /// Send a transaction message from the default user_id of the client to the given receiver_user_id, e.g, Send(`receiver_user_id`, `transaction_message`)
    Send(String, String),
    /// Wait for the given number of milliseconds, e.g., SleepMs(`milliseconds`)
    SleepMs(u64),
}

/// Read a file and return the content as a string.
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath)
        .expect(&("Cannot read ".to_owned() + filepath));
    contents
}

/// A flag indicating whether to disable the UI thread if you need to check some debugging outputs that is covered by the UI. 
/// Eventually this should be set to false and you shouldn't output debugging information directly to stdout or stderr.
const NO_UI_DEBUG_NODE: bool = false;

fn main() {
    // The usage of bin_client is as follows:
    // bin_client <client_seccomp_path> <nakamoto_config_path> <nakamoto_seccomp_path> <wallet_config_path> <wallet_seccomp_path> [<bot_command_path>]
    // - `client_seccomp_path`: The path to the seccomp file for this client process for Part B. (You can set this argument to any value during Part A.)
    // - `nakamoto_config_path`: The path to the config folder for the bin_nakamoto process. For example, `./tests/nakamoto_config1`. Your program should read the 3 files in the config folder (`BlockTree.json`, `Config.json`, `TxPool.json`) for initializing bin_nakamoto.
    // - `nakamoto_seccomp_path`: The path to the seccomp file for the bin_nakamoto process for Part B. (You can set this argument to any value during Part A.)
    // - `wallet_config_path`: The path to the config file for the bin_wallet process. For example, `./tests/_secrets/Walley.A.json`. Your program should read the file for initializing bin_wallet.
    // - `wallet_seccomp_path`: The path to the seccomp file for the bin_wallet process for Part B. (You can set this argument to any value during Part A.)
    // - [`bot_command_path`]: *Optional* argument. The path to the file or named pipe for the bot commands. If this argument is provided, your program should read commands line-by-line from the file.
    //                         an example file of the bot commands can be found at `./tests/_bots/botA-0.jsonl`. You can also look at `run_four.sh` for an example of using the named pipe version of this argument.
    //                         The bot commands are executed by the client in the order they are read from the file or the named pipe. 
    //                         The bot commands should be executed in a separate thread so that the UI thread can still be responsive.
    // Please fill in the blank

    let args: Vec<String> = std::env::args().collect();
    let num_args = args.len();

    let client_seccomp_path = std::env::args().nth(1).expect("Please specify client seccomp path");
    let nakamoto_config_path = std::env::args().nth(2).expect("Please specify nakamoto config path");
    let nakamoto_seccomp_path = std::env::args().nth(3).expect("Please specify nakamoto seccomp path");
    let wallet_config_path = std::env::args().nth(4).expect("Please specify wallet config path");
    let wallet_seccomp_path = std::env::args().nth(5).expect("Please specify wallet seccomp path");

    // - Create bin_nakamoto process:  Command::new("./target/debug/bin_nakamoto")...
    let mut nakamoto_process = Command::new("./target/debug/bin_nakamoto").arg(nakamoto_seccomp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute bin_nakamoto as child process");

    // - Create bin_wallet process:  Command::new("./target/debug/bin_wallet")...
    let mut bin_wallet_process = Command::new("./target/debug/bin_wallet").arg(wallet_seccomp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute bin_wallet as child process");

    // - Get stdin and stdout of those processes
    // - Create buffer readers if necessary
    let mut nakamoto_process_stdin = nakamoto_process.stdin.take().expect("Failed to get nakamoto_child stdin");
    let nakamoto_process_stdout = nakamoto_process.stdout.take().expect("Failed to get nakamoto_child stdout");
    let nakamoto_process_stderr = nakamoto_process.stderr.take().expect("Failed to get nakamoto_child stderr");
    
    let mut bin_wallet_process_stdin = bin_wallet_process.stdin.take().expect("Failed to get wallet_child stdin");
    let bin_wallet_process_stdout = bin_wallet_process.stdout.take().expect("Failed to get wallet_child stdout");
    let bin_wallet_process_stderr = bin_wallet_process.stderr.take().expect("Failed to get wallet_child stderr");
    
    let bin_wallet_reader =  Arc::new(Mutex::new(BufReader::new(bin_wallet_process_stdout)));

    // - Send initialization requests to bin_nakamoto and bin_wallet
    let nakamoto_config_configpath = format!("{}/Config.json", nakamoto_config_path);
    let nakamoto_config_blocktreepath = format!("{}/BlockTree.json", nakamoto_config_path);
    let nakamoto_config_txpoolpath = format!("{}/TxPool.json", nakamoto_config_path);

    let nakamoto_init_config_msg = read_string_from_file(nakamoto_config_configpath.as_str());
    let nakamoto_init_blocktree_msg = read_string_from_file(nakamoto_config_blocktreepath.as_str());
    let nakamoto_init_txpool_msg = read_string_from_file(nakamoto_config_txpoolpath.as_str());
    let nakamoto_init_msg_serialised = serde_json::to_string(
        &IPCMessageReqNakamoto::Initialize(nakamoto_init_blocktree_msg, nakamoto_init_txpool_msg, nakamoto_init_config_msg)).unwrap();
    nakamoto_process_stdin.write_all(format!("{}\n", nakamoto_init_msg_serialised).as_bytes()).unwrap();

    // bin_wallet initalisation
    let wallet_init_msg = read_string_from_file(wallet_config_path.as_str());
    let wallet_init_msg_serialised = serde_json::to_string(&IPCMessageReqWallet::Initialize(wallet_init_msg)).unwrap();
    bin_wallet_process_stdin.write_all(format!("{}\n", wallet_init_msg_serialised).as_bytes()).unwrap();

    // sandboxing the bin_client (For part B). Leave it blank for part A.
    let maybe_policy_path = std::env::args().nth(1);
    if let Some(policy_path) = maybe_policy_path {
        // Please fill in the blank
        // If the first param is provided, read the seccomp config and apply it
        let json_input = read_string_from_file(&policy_path);
        let filter_map: BpfMap = seccompiler::compile_from_json(
            json_input.as_bytes(),
            std::env::consts::ARCH.try_into().unwrap(),
        )
        .unwrap();
        let filter = filter_map.get("main_thread").unwrap() ;
        seccompiler::apply_filter(filter).unwrap();
    }

    // Please fill in the blank
    // Clear reply from bin_wallet process.
    let mut wallet_info_output = String::new();
    bin_wallet_reader.lock().unwrap().read_line(&mut wallet_info_output).expect("Failed to retrieve wallet user info");
    let wallet_info_one = serde_json::from_str(&wallet_info_output).unwrap();
    match wallet_info_one {
        IPCMessageRespWallet::Initialized => {
            wallet_info_output = "".to_string();
        },
        _=> { 
            panic!() 
        }
    };
    // Read the user info from wallet
    let user_name: String;
    let user_id: String;

    let wallet_get_user_info_msg_serialised = serde_json::to_string(&IPCMessageReqWallet::GetUserInfo).unwrap();
    bin_wallet_process_stdin.write_all(format!("{}\n", wallet_get_user_info_msg_serialised).as_bytes()).unwrap();

    bin_wallet_reader.lock().unwrap().read_line(&mut wallet_info_output).expect("Failed to retrieve wallet user info");
    let wallet_info = serde_json::from_str(&wallet_info_output).unwrap();
    //println!("{}", wallet_info_output);
    match wallet_info {
        IPCMessageRespWallet::UserInfo(username, userid) => {
            user_name = username.clone();
            user_id = userid.clone();
        },
        _=> { 
            panic!() 
        }
    };

    // Create the Terminal UI app
    let app_arc = Arc::new(Mutex::new(app::App::new(
        user_name.clone(), 
        user_id.clone(), 
        "".to_string(), 
        format!("SEND $100   // By {}", user_name)))
    );

    // An enclosure func to generate signing requests when creating new transactions. 
    let create_sign_req = |sender: String, receiver: String, message: String| {
        let timestamped_message = format!("{}   // {}", message, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
        let sign_req = IPCMessageReqWallet::SignRequest(serde_json::to_string(&(sender, receiver, timestamped_message)).unwrap());
        let mut sign_req_str = serde_json::to_string(&sign_req).unwrap();
        sign_req_str.push('\n');
        return sign_req_str;
    };

    // Arc mutex starts here
    let nakamoto_status_stdin_p = Arc::new(Mutex::new(nakamoto_process_stdin)); // nakamoto std in
    let nakamoto_stdin_p = Arc::clone(&nakamoto_status_stdin_p); // nakamoto std in for ui
    let nakamoto_status_reader = Arc::new(Mutex::new(BufReader::new(nakamoto_process_stdout))); // nakamoto std out

    let bin_wallet_stdin_p = Arc::new(Mutex::new(bin_wallet_process_stdin));
    // bin_wallet std out, bin_wallet_reader on top

    if std::env::args().len() != 6 {
        // Then there must be 7 arguments provided. The last argument is the bot commands path
        // Please fill in the blank
        // Create a thread to read the bot commands from `bot_command_path`, execute those commands and update the UI
        // Notice that the `SleepMs(1000)` doesn't mean that the all threads in the whole process should sleep for 1000ms. It means that 
        // The next bot command that fakes the user interaction should be processed 1000ms later. 
        // It should not block the execution of any other threads or the main thread.
        let bot_command_path = std::env::args().nth(6).expect("Failed to get bot_command_path");
        let bot_command_file = File::open(bot_command_path).expect("Failed to open bot_cmd_file");
        let bot_command_reader = BufReader::new(bot_command_file);
        let bin_wallet_stdin_p_clone = bin_wallet_stdin_p.clone();
        let user_id_clone = user_id.to_string();
        thread::spawn(move || {
            for bot_line in  bot_command_reader.lines() {
                let bot_line = bot_line.expect("Failed to read line");
                let parsed_bot_cmd : BotCommand = serde_json::from_str(bot_line.as_str()).unwrap(); 
                match parsed_bot_cmd {
                    BotCommand::Send(receiver_user_id, transaction_message) => {
                        let sign_request = create_sign_req(user_id_clone.to_string(), receiver_user_id, transaction_message);
                        //Add to Tx Pool
                        // publish sign request to stdin to wallet
                        bin_wallet_stdin_p_clone.lock().unwrap().write_all(sign_request.as_bytes()).unwrap();
                    }
                    BotCommand::SleepMs(sleeptime) => {
                        thread::sleep(Duration::from_millis(sleeptime));
                        //good night
                    }
                    _ => {
                        //Do nth
                    }
                }    
            }
        });
    }

    // Please fill in the blank
    // - Spawn threads to read/write from/to bin_nakamoto/bin_wallet. (Through their piped stdin and stdout)
    // - You should request for status update from bin_nakamoto periodically (every 500ms at least) to update the App (UI struct) accordingly.
    // - You can also create threads to read from stderr of bin_nakamoto/bin_wallet and add those lines to the UI (app.stderr_log) for easier debugging.

    let nakamoto_status_stdin_p_clone = Arc::clone(&nakamoto_status_stdin_p);
    let user_id_clone = user_id.clone();
    let nakamote_write_app = app_arc.clone();
    let nakamoto_write_threads = thread::spawn(move || {
        let tick_rate = Duration::from_millis(3000);
        loop {
            // Get status update
            let status_update_msg = serde_json::to_string(&IPCMessageReqNakamoto::RequestChainStatus).unwrap();
            nakamoto_status_stdin_p_clone.lock().unwrap().write_all(format!("{}\n", status_update_msg).as_bytes()).unwrap();

            let status_update_msg = serde_json::to_string(&IPCMessageReqNakamoto::RequestMinerStatus).unwrap();
            nakamoto_status_stdin_p_clone.lock().unwrap().write_all(format!("{}\n", status_update_msg).as_bytes()).unwrap();

            let status_update_msg = serde_json::to_string(&IPCMessageReqNakamoto::RequestNetStatus).unwrap();
            nakamoto_status_stdin_p_clone.lock().unwrap().write_all(format!("{}\n", status_update_msg).as_bytes()).unwrap();

            let status_update_msg = serde_json::to_string(&IPCMessageReqNakamoto::RequestTxPoolStatus).unwrap();
            nakamoto_status_stdin_p_clone.lock().unwrap().write_all(format!("{}\n", status_update_msg).as_bytes()).unwrap();

            let status_update_msg = serde_json::to_string(&IPCMessageReqNakamoto::GetAddressBalance(user_id_clone.to_string())).unwrap();
            nakamoto_status_stdin_p_clone.lock().unwrap().write_all(format!("{}\n", status_update_msg).as_bytes()).unwrap();

            if nakamote_write_app.lock().unwrap().should_quit {
                break;
            }
            thread::sleep(tick_rate);
        }
    });

    let bin_wallet_reader_clone = Arc::clone(&bin_wallet_reader);
    let nakamoto_status_stdin_p_clone_clone = Arc::clone(&nakamoto_status_stdin_p);
    let wallet_read_threads = thread::spawn(move || {
        loop {
            let mut wallet_sign_response = String::new();
            bin_wallet_reader_clone.lock().unwrap().read_line(&mut wallet_sign_response).expect("Failed to retrieve wallet user info");
            let sign_response = serde_json::from_str(&wallet_sign_response).unwrap();
            match sign_response {
                IPCMessageRespWallet::SignResponse(data_string, signature) => {
                    // eprintln!("found a sign response! publishing to tx...");
                    let publish_tx_req = serde_json::to_string(&IPCMessageReqNakamoto::PublishTx(data_string, signature)).unwrap();
                    nakamoto_status_stdin_p_clone_clone.lock().unwrap().write_all(format!("{}\n", publish_tx_req).as_bytes()).unwrap();
                },
                _=> {}
            };
        }
    });

    let nakamoto_read_app = app_arc.clone();
    let nakamoto_read_threads = thread::spawn(move || {
        loop {
            let mut msg = String::new();
            nakamoto_status_reader.lock().unwrap().read_line(&mut msg).unwrap();
            let msg = serde_json::from_str(msg.as_str()).unwrap();
            match msg {
                IPCMessageRespNakamoto::ChainStatus(status) => {
                    nakamoto_read_app.lock().unwrap().blocktree_status = status;
                },
                IPCMessageRespNakamoto::NetStatus(status) => {
                    nakamoto_read_app.lock().unwrap().network_status = status;
                },
                IPCMessageRespNakamoto::TxPoolStatus(status) => {
                    nakamoto_read_app.lock().unwrap().txpool_status = status;
                },
                IPCMessageRespNakamoto::MinerStatus(status) => {
                    nakamoto_read_app.lock().unwrap().miner_status = status;
                },
                IPCMessageRespNakamoto::AddressBalance(_, balance) => {
                    nakamoto_read_app.lock().unwrap().user_balance = balance;
                },
                IPCMessageRespNakamoto::Notify(msg) => {
                    nakamoto_read_app.lock().unwrap().notify_log.push(msg);
                },
                IPCMessageRespNakamoto::StateSerialization(chain_info, txpool_info) => {
                    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                    let block_tree_path = format!("{}/{}-BlockTree.json", nakamoto_config_path, timestamp);
                    let tx_pool_path = format!("{}/{}-TxPool.json", nakamoto_config_path, timestamp);
                    nakamoto_read_app.lock().unwrap().notify_log.push(block_tree_path.clone());
                    match fs::write(block_tree_path, chain_info) {
                        Ok(_) => { nakamoto_read_app.lock().unwrap().notify_log.push("[File] BlockTree.json saved".to_string()); },
                        Err(_) => { nakamoto_read_app.lock().unwrap().notify_log.push("[File] Error saving BlockTree.json".to_string()); },
                    };
                    match fs::write(tx_pool_path, txpool_info) {
                        Ok(_) => { nakamoto_read_app.lock().unwrap().notify_log.push("[File] TxPool.json saved".to_string()); },
                        Err(_) => { nakamoto_read_app.lock().unwrap().notify_log.push("[File] Error saving TxPool.json".to_string()); }
                    };
                }
                _=> {}
            }

            if nakamoto_read_app.lock().unwrap().should_quit {
                break;
            }
        }
    });

    let err_app = app_arc.clone();
    let err_thread = thread::spawn(move || {
        let mut nakamoto_err_reader = BufReader::new(nakamoto_process_stderr);
        loop {
            let mut err_msg = String::new();

            match nakamoto_err_reader.read_line(&mut err_msg) {
                Ok(_) => { err_app.lock().unwrap().stderr_log.push(err_msg); },
                Err(_) => {},
            }

            if err_app.lock().unwrap().should_quit {
                break;
            }

            thread::sleep(Duration::from_millis(1000));
        }
    });

    let err_app2 = app_arc.clone();
    let err_thread2 = thread::spawn(move || {
        let mut wallet_err_reader = BufReader::new(bin_wallet_process_stderr);
        loop {
            let mut err_msg2 = String::new();

            match wallet_err_reader.read_line(&mut err_msg2) {
                Ok(_) => { err_app2.lock().unwrap().stderr_log.push(err_msg2); },
                Err(_) => {},
            }

            if err_app2.lock().unwrap().should_quit {
                break;
            }

            thread::sleep(Duration::from_millis(1000));
        }
    });

    // UI thread. Modify it to suit your needs. 
    let app_ui_ref = app_arc.clone();
    let bin_wallet_stdin_p_cloned = bin_wallet_stdin_p.clone();
    let nakamoto_stdin_p_cloned = nakamoto_stdin_p.clone();
    let handle_ui = thread::spawn(move || {
        let tick_rate = Duration::from_millis(200);
        if NO_UI_DEBUG_NODE {
            // If app_ui.should_quit is set to true, the UI thread will exit.
            loop {
                if app_ui_ref.lock().unwrap().should_quit {
                    break;
                }
                // sleep for 500ms
                thread::sleep(Duration::from_millis(500));
            }
            return;
        }
        let ui_loop = || -> Result<(), io::Error> {
            // setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let mut last_tick = Instant::now();
            loop {
                terminal.draw(|f| {
                    app_ui_ref.lock().unwrap().draw(f)
                })?;

                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_millis(100));
                
                if crossterm::event::poll(timeout)? {
                    let input = event::read()?.into();
                    let mut app = app_ui_ref.lock().unwrap();
                    match input {
                        Input { key: Key::Esc, .. } => {app.on_quit();}
                        Input { key: Key::Down, .. } => {app.on_down()}
                        Input { key: Key::Up, .. } => {app.on_up()},
                        Input { key: Key::Enter, .. } => {
                            if !app.are_inputs_valid {
                                app.client_log("Invalid inputs! Cannot create Tx.".to_string());
                            } else {
                                let (sender, receiver, message) = app.on_enter();
                                let sign_req_str = create_sign_req(sender, receiver, message);
                                bin_wallet_stdin_p_cloned.lock().unwrap().write_all(sign_req_str.as_bytes()).unwrap();
                            }
                        }
                        // on control + s, request Nakamoto to serialize its state
                        Input { key: Key::Char('s'), ctrl: true, .. } => {
                            let serialize_req = IPCMessageReqNakamoto::RequestStateSerialization;
                            let mut nakamoto_stdin = nakamoto_stdin_p_cloned.lock().unwrap();
                            let mut to_send = serde_json::to_string(&serialize_req).unwrap();
                            to_send.push_str("\n");
                            nakamoto_stdin.write_all(to_send.as_bytes()).unwrap();
                        }
                        input => {
                            app.on_textarea_input(input);
                        }
                    }
                }

                let mut app = app_ui_ref.lock().unwrap();
                if last_tick.elapsed() >= tick_rate {
                    app.on_tick();
                    last_tick = Instant::now();
                }
                if app.should_quit {
                    break;
                }
            }
            // restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            Ok(())
        };
        ui_loop().unwrap();
    }); 
    handle_ui.join().unwrap();
    
    eprintln!("--- Sending \"Quit\" command...");
    nakamoto_stdin_p.lock().unwrap().write_all("\"Quit\"\n".as_bytes()).unwrap();
    bin_wallet_stdin_p.lock().unwrap().write_all("\"Quit\"\n".as_bytes()).unwrap();

    // Please fill in the blank
    // Wait for the IPC threads to finish
    nakamoto_read_threads.join().unwrap();
    nakamoto_write_threads.join().unwrap();
    wallet_read_threads.join().unwrap();
    err_thread.join().unwrap();
    err_thread2.join().unwrap();

    let ecode1 = nakamoto_process.wait().expect("failed to wait on child nakamoto");
    eprintln!("--- nakamoto ecode: {}", ecode1);

    let ecode2 = bin_wallet_process.wait().expect("failed to wait on child bin_wallet");
    eprintln!("--- bin_wallet ecode: {}", ecode2);
    
}

