package org.example;

public class Main {
    public static void main(String[] args) throws InterruptedException {
        System.out.println("Pre!");
        System.load(args[0]);
        System.out.println("Loaded!");
        JNITest.test();
        System.out.println("Done!");
        Thread.sleep(5000);
        System.out.println("Exiting!");
        System.exit(0);

    }
}