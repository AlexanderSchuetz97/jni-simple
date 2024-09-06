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

        System.err.println("staticVoidMethod3 C=" + c);
    }

    public static Object staticObjectMethod0() {
        reset();
        name = "staticObjectMethod0";
        return new Object();
    }

    public static Object staticObjectMethod1(short a) {
        reset();
        name = "staticObjectMethod1";
        MethodCalls.a = a;
        return new Object();
    }

    public static Object staticObjectMethod2(short a, Object b) {
        reset();
        name = "staticObjectMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return new Object();
    }

    public static Object staticObjectMethod3(short a, Object b, double c) {
        reset();
        name = "staticObjectMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return new Object();
    }

    public static boolean staticBooleanMethod0() {
        reset();
        name = "staticBooleanMethod0";
        return true;
    }

    public static boolean staticBooleanMethod1(short a) {
        reset();
        name = "staticBooleanMethod1";
        MethodCalls.a = a;
        return true;
    }

    public static boolean staticBooleanMethod2(short a, Object b) {
        reset();
        name = "staticBooleanMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return true;
    }

    public static boolean staticBooleanMethod3(short a, Object b, double c) {
        reset();
        name = "staticBooleanMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return true;
    }

    public static byte staticByteMethod0() {
        reset();
        name = "staticByteMethod0";
        return 1;
    }

    public static byte staticByteMethod1(short a) {
        reset();
        name = "staticByteMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public static byte staticByteMethod2(short a, Object b) {
        reset();
        name = "staticByteMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public static byte staticByteMethod3(short a, Object b, double c) {
        reset();
        name = "staticByteMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public static char staticCharMethod0() {
        reset();
        name = "staticCharMethod0";
        return 1;
    }

    public static char staticCharMethod1(short a) {
        reset();
        name = "staticCharMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public static char staticCharMethod2(short a, Object b) {
        reset();
        name = "staticCharMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public static char staticCharMethod3(short a, Object b, double c) {
        reset();
        name = "staticCharMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public static short staticShortMethod0() {
        reset();
        name = "staticShortMethod0";
        return 1;
    }

    public static short staticShortMethod1(short a) {
        reset();
        name = "staticShortMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public static short staticShortMethod2(short a, Object b) {
        reset();
        name = "staticShortMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public static short staticShortMethod3(short a, Object b, double c) {
        reset();
        name = "staticShortMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        System.err.println("staticShortMethod3 C=" + c);
        return 1;
    }

    public static int staticIntMethod0() {
        reset();
        name = "staticIntMethod0";
        return 1;
    }

    public static int staticIntMethod1(short a) {
        reset();
        name = "staticIntMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public static int staticIntMethod2(short a, Object b) {
        reset();
        name = "staticIntMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public static int staticIntMethod3(short a, Object b, double c) {
        reset();
        name = "staticIntMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public static long staticLongMethod0() {
        reset();
        name = "staticLongMethod0";
        return 1;
    }

    public static long staticLongMethod1(short a) {
        reset();
        name = "staticLongMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public static long staticLongMethod2(short a, Object b) {
        reset();
        name = "staticLongMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public static long staticLongMethod3(short a, Object b, double c) {
        reset();
        name = "staticLongMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public static float staticFloatMethod0() {
        reset();
        name = "staticFloatMethod0";
        return 1;
    }

    public static float staticFloatMethod1(short a) {
        reset();
        name = "staticFloatMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public static float staticFloatMethod2(short a, Object b) {
        reset();
        name = "staticFloatMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public static float staticFloatMethod3(short a, Object b, double c) {
        reset();
        name = "staticFloatMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public static double staticDoubleMethod0() {
        reset();
        name = "staticDoubleMethod0";
        return 1;
    }

    public static double staticDoubleMethod1(short a) {
        reset();
        name = "staticDoubleMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public static double staticDoubleMethod2(short a, Object b) {
        reset();
        name = "staticDoubleMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public static double staticDoubleMethod3(short a, Object b, double c) {
        reset();
        name = "staticDoubleMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }
}