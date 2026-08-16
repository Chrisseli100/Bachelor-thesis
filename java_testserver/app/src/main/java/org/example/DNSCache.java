package org.example;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.concurrent.ConcurrentHashMap;

public class DNSCache {
    private ConcurrentHashMap<String, String> cache;

    public DNSCache() {
        this.cache = new ConcurrentHashMap<>();
    }

    //search inside the cache first and the file after if needed
    public String getIp(String domain) {
        String ip = cache.computeIfAbsent(domain, d -> {
            String prefixString = d + " ";
            try(BufferedReader buffer = new BufferedReader(new FileReader("generated_hosts.txt"))) {
                String line;
                while ((line = buffer.readLine()) != null) {
                    if(line.startsWith(prefixString)) {
                        String ipAdresse = line.substring(prefixString.length());
                        return ipAdresse;
                    }
                }
                return "NODATA";
            } catch (IOException e) {
                e.printStackTrace();
                return "NODATA";
            }
        });
        return ip;
    }
}
