package org.example;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public class Server {
    private ServerSocket serverSocket;
    private DNSCache cache;

    //constructor with new cache
    public Server(int port) {
        try {
            this.serverSocket = new ServerSocket(port);
            this.cache = new DNSCache();
        } catch(IOException e) {
            System.err.println("Couldnt create server.");
            e.printStackTrace();
        }
    }

    //overloaded constructor to share a cache
    public Server(int port, DNSCache cache) {
        try {
            this.serverSocket = new ServerSocket(port);
            this.cache = cache;
        } catch(IOException e) {
            System.err.println("Couldnt create server.");
            e.printStackTrace();
        }
    }

    //listen for requests and forward them to the threadpool
    public void run(int poolSize) {
        ExecutorService threadPool = Executors.newFixedThreadPool(poolSize);

        while(true) {
            try {
                Socket clientSocket = serverSocket.accept();
                threadPool.execute(() -> this.handleConnection(clientSocket));
            } catch (IOException e) {
                System.err.println("Error connecting with client.");
                e.printStackTrace();
            }
        }
    }

    private void handleConnection(Socket clientSocket) {
        try {
            BufferedReader buffer = new BufferedReader(new InputStreamReader(clientSocket.getInputStream()));

            String requestLine = buffer.readLine();
            String statusLine;
            String domain = String.valueOf("");

            //read the request-line and set status-line/domain
            //filter for bad requests
            if(requestLine.contains("GET /dns-query") && requestLine.contains("type=A")) {
                domain = requestLine.split("name=")[1].split("&type=A")[0];
                statusLine = "HTTP/1.1 200 OK";
            } else {
                statusLine = "HTTP/1.1 400 BAD REQUEST";
            }

            //respond to bad request and do no further work
            if(domain.isBlank()) {
                respondToClient(clientSocket, statusLine, "", 0);
                return;
            }

            //find correct ip
            String ip = cache.getIp(domain);

            int contentLength = 0;

            //read the rest of the http headers
            //if additional headers must be searched/found, add them in the loop
            String lineBuffer;
            while (true) {
                lineBuffer = buffer.readLine();

                if(lineBuffer.isEmpty()) {
                    break;
                }

                if(lineBuffer.split(" ")[0].trim().equalsIgnoreCase("Content-Length:")) {
                    contentLength = Integer.parseInt(lineBuffer.split(" ")[1].trim());
                }
            }

            //calculate the rest of the data needed for a correct response
            char[] content = new char[contentLength];
            buffer.read(content, 0, contentLength);
            String requestNumber = new String(content);

            String body = ip + "," + requestNumber;
            int bodySize = body.getBytes(StandardCharsets.UTF_8).length;

            respondToClient(clientSocket, statusLine, body, bodySize);
            buffer.close();

        } catch(IOException e) {
            e.printStackTrace();
            this.respondToClient(clientSocket, "HTTP/1.1 500 INTERNAL SERVER ERROR", "", 0);
        }
    }

    private void respondToClient(Socket clientSocket, String statusLine, String body, int bodySize) {
        String response = statusLine + "\r\nContent-Length: " + bodySize + "\r\n\r\n" + body;
        try {
            PrintWriter outgoing = new PrintWriter(clientSocket.getOutputStream(), true);
            outgoing.write(response);
            outgoing.flush();
            outgoing.close();
            clientSocket.close();
        } catch (IOException e) {
            System.err.println("Failed to send answer to client.");
            e.printStackTrace();
        }
    }
}
