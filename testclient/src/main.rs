use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::time::Instant;
use std::net::TcpStream;

fn main() {
    //read/create configuration and print it to the user
    let configuration = Configuration::new();
    println!("---Current Configuration---\nServeradress: {}\nRequestfile: {}\nSend incorrect request: {}\n"
    , configuration.server_adress, configuration.requestfile, configuration.send_incorrect_request);

    //read requestfile and prepare the loadtest
    let mut requests: Vec<&str> = Vec::new();
    let request_content = fs::read_to_string(&configuration.requestfile)
        .expect("Couldnt find/read the requestfile");
    for lines in request_content.lines() {
        requests.push(lines.trim());
    }

    //wait for user to start the test
    wait_for_user();

    let mut counter = 1;
    let mut result_times = HashMap::new();

    let timer = Instant::now();

    for request in requests {
        let mut stream = TcpStream::connect(&configuration.server_adress).expect("Couldnt connect to server");

        let counter_string = counter.to_string();
        let content_length = counter_string.len();

        let message_content = 
            format!("GET /dns-query?name={request}&type=A HTTP/1.1\r\nHost: {}\r\nContent-Length: {content_length}\r\n\r\n{counter}"
            , configuration.server_adress.split(':').nth(0).unwrap());

        //use the line below to print the send message, only for debug purposes
        //println!("{}\n", message_content);
        stream.write(message_content.as_bytes()).expect("Couldnt send message");

        let responsetimer = Instant::now();

        let mut buf = [0; 150];
        stream.read(&mut buf).unwrap();

        //use the lines below to print the serverresponse, only for debug purposes
        //println!("-----------------------");
        //let answer = String::from_utf8_lossy(&buf);
        //println!("{}", answer);

        let responsetime = responsetimer.elapsed().as_micros();
        result_times.insert(counter, responsetime);

        counter += 1;
    }

    if configuration.send_incorrect_request == true {
        test_incorrect_request(configuration);
    }

    //show testresults and save them to a file
    let elapsed = timer.elapsed();
    println!("Test complete!");
    println!("Testduration: {:.2?}", elapsed);
    println!("Creating resultfile...");

    let mut result_file = File::create("results.csv").expect("cannot create the resultfile");
    for n in 1..counter {
        let result= format!("{},{}\n", n, result_times.get(&n).unwrap());
        let file_input = result.as_bytes();
        result_file.write_all(&file_input).expect("couldnt write into the resultfile");
    }
}

struct Configuration {
    server_adress: String,
    requestfile: String,
    send_incorrect_request: bool,
}


impl Configuration {
    fn new() -> Configuration {
        let mut server_ip= "";
        let mut server_port= "";
        let mut requestfile = String::new();
        let mut send_incorrect_request = false;

        let configfile = fs::read_to_string("configurationfile.txt").expect("no configurationfile");

        for line in configfile.lines() {
            if line.contains("server_ip") {
                server_ip = line.split(':').nth(1)
                .expect("expected a server_ip in configurationfile")
                .trim();
                continue;
            }
            if line.contains("server_port") {
                server_port = line.split(':').nth(1)
                .expect("expected a server_port in configurationfile")
                .trim();
                continue;
            }
            if line.contains("requestfile") {
                requestfile = line.split(':').nth(1)
                .expect("expected a requestfile in configurationfile")
                .trim().to_string();
                continue;
            }
            if line.contains("send_incorrect_request") {
                send_incorrect_request = line.split(':').nth(1)
                .expect("expected a requestfile in configurationfile")
                .trim().parse()
                .expect("expected a boolean for send_incorrect_requestin configurationfile");
                continue;
            }
        }

        let server_adress = format!("{server_ip}:{server_port}");

        Configuration {
            server_adress,
            requestfile,
            send_incorrect_request,
        }
    }
}

fn wait_for_user() {
    loop {
        print!("Rquests are loaded and ready. Type 'start' for starting the benchmarking process: ");
        io::stdout().flush().expect("flushing error");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().eq_ignore_ascii_case("start") {
            false => continue,
            true => {
                println!("Starting loadtest...");
                break;
            }
        }
    }
}

fn test_incorrect_request(configuration: Configuration) {
    let mut stream = TcpStream::connect(&configuration.server_adress).expect("Couldnt connect to server");
    stream.write("POST /example HTTP/1.1\r\n".as_bytes()).expect("Couldnt send message");
    println!("sending incorrect_request");

    //use the lines below to show the serverresponse to the incorrect request, only for debug purposes
    /*let mut buf = [0; 128];
    stream.read(&mut buf).unwrap();
    let answer = String::from_utf8_lossy(&buf);
    println!("{}", answer);*/
}
