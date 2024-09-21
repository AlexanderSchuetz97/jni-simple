import java.lang.System;

public class RegisterTest {
    public static native void test(String abc);

    public static native void test(double abc);

    public static void callTest(String abc) {
        test(abc);
    }

    public static void callTest(double abc) {
        test(abc);
    }
}