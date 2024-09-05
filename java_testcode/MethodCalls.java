import java.lang.System;

public class MethodCalls {

    public static volatile String name;
    public static volatile short a;
    public static volatile Object b;
    public static volatile double c;

    public static void reset() {
        a = 0;
        b = null;
        c = 0;
        name = null;
    }

    public static void staticVoidMethod0() {
        reset();
        name = "staticVoidMethod0";
    }

    public static void staticVoidMethod1(short a) {
        reset();
        name = "staticVoidMethod1";
        MethodCalls.a = a;
    }

    public static void staticVoidMethod2(short a, Object b) {
        reset();
        name = "staticVoidMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
    }

    public static void staticVoidMethod3(short a, Object b, double c) {
        reset();
        name = "staticVoidMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
    }

}