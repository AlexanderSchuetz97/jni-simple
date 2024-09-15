import java.lang.System;

public class MethodCalls {


    public static volatile String name;
    public static volatile short a;
    public static volatile Object b;
    public static volatile double c;


    public static class NvChild extends MethodCalls {
        public void dynVoidMethod0() {
            reset();
            name = "nvVoidMethod0";
        }

        public void dynVoidMethod1(short a) {
            reset();
            name = "nvVoidMethod1";
            MethodCalls.a = a;
        }

        public void dynVoidMethod2(short a, Object b) {
            reset();
            name = "nvVoidMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
        }

        public void dynVoidMethod3(short a, Object b, double c) {
            reset();
            name = "nvVoidMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;

            System.err.println("dynVoidMethod3 C=" + c);
        }

        public Object dynObjectMethod0() {
            reset();
            name = "nvObjectMethod0";
            return new Object();
        }

        public Object dynObjectMethod1(short a) {
            reset();
            name = "nvObjectMethod1";
            MethodCalls.a = a;
            return new Object();
        }

        public Object dynObjectMethod2(short a, Object b) {
            reset();
            name = "nvObjectMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return new Object();
        }

        public Object dynObjectMethod3(short a, Object b, double c) {
            reset();
            name = "nvObjectMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            return new Object();
        }

        public boolean dynBooleanMethod0() {
            reset();
            name = "nvBooleanMethod0";
            return true;
        }

        public boolean dynBooleanMethod1(short a) {
            reset();
            name = "nvBooleanMethod1";
            MethodCalls.a = a;
            return true;
        }

        public boolean dynBooleanMethod2(short a, Object b) {
            reset();
            name = "nvBooleanMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return true;
        }

        public boolean dynBooleanMethod3(short a, Object b, double c) {
            reset();
            name = "nvBooleanMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            return true;
        }

        public byte dynByteMethod0() {
            reset();
            name = "nvByteMethod0";
            return 1;
        }

        public byte dynByteMethod1(short a) {
            reset();
            name = "nvByteMethod1";
            MethodCalls.a = a;
            return 1;
        }

        public byte dynByteMethod2(short a, Object b) {
            reset();
            name = "nvByteMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return 1;
        }

        public byte dynByteMethod3(short a, Object b, double c) {
            reset();
            name = "nvByteMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            return 1;
        }

        public char dynCharMethod0() {
            reset();
            name = "nvCharMethod0";
            return 1;
        }

        public char dynCharMethod1(short a) {
            reset();
            name = "nvCharMethod1";
            MethodCalls.a = a;
            return 1;
        }

        public char dynCharMethod2(short a, Object b) {
            reset();
            name = "nvCharMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return 1;
        }

        public char dynCharMethod3(short a, Object b, double c) {
            reset();
            name = "nvCharMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            return 1;
        }

        public short dynShortMethod0() {
            reset();
            name = "nvShortMethod0";
            return 1;
        }

        public short dynShortMethod1(short a) {
            reset();
            name = "nvShortMethod1";
            MethodCalls.a = a;
            return 1;
        }

        public short dynShortMethod2(short a, Object b) {
            reset();
            name = "nvShortMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return 1;
        }

        public short dynShortMethod3(short a, Object b, double c) {
            reset();
            name = "nvShortMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            System.err.println("dynShortMethod3 C=" + c);
            return 1;
        }

        public int dynIntMethod0() {
            reset();
            name = "nvIntMethod0";
            return 1;
        }

        public int dynIntMethod1(short a) {
            reset();
            name = "nvIntMethod1";
            MethodCalls.a = a;
            return 1;
        }

        public int dynIntMethod2(short a, Object b) {
            reset();
            name = "nvIntMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return 1;
        }

        public int dynIntMethod3(short a, Object b, double c) {
            reset();
            name = "nvIntMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            return 1;
        }

        public long dynLongMethod0() {
            reset();
            name = "nvLongMethod0";
            return 1;
        }

        public long dynLongMethod1(short a) {
            reset();
            name = "nvLongMethod1";
            MethodCalls.a = a;
            return 1;
        }

        public long dynLongMethod2(short a, Object b) {
            reset();
            name = "nvLongMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return 1;
        }

        public long dynLongMethod3(short a, Object b, double c) {
            reset();
            name = "nvLongMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            return 1;
        }

        public float dynFloatMethod0() {
            reset();
            name = "nvFloatMethod0";
            return 1;
        }

        public float dynFloatMethod1(short a) {
            reset();
            name = "nvFloatMethod1";
            MethodCalls.a = a;
            return 1;
        }

        public float dynFloatMethod2(short a, Object b) {
            reset();
            name = "nvFloatMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return 1;
        }

        public float dynFloatMethod3(short a, Object b, double c) {
            reset();
            name = "nvFloatMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            return 1;
        }

        public double dynDoubleMethod0() {
            reset();
            name = "nvDoubleMethod0";
            return 1;
        }

        public double dynDoubleMethod1(short a) {
            reset();
            name = "nvDoubleMethod1";
            MethodCalls.a = a;
            return 1;
        }

        public double dynDoubleMethod2(short a, Object b) {
            reset();
            name = "nvDoubleMethod2";
            MethodCalls.a = a;
            MethodCalls.b = b;
            return 1;
        }

        public double dynDoubleMethod3(short a, Object b, double c) {
            reset();
            name = "nvDoubleMethod3";
            MethodCalls.a = a;
            MethodCalls.b = b;
            MethodCalls.c = c;
            return 1;
        }
    }

    public static void reset() {
        a = 0;
        b = null;
        c = 0;
        name = null;
    }

    public MethodCalls() {
        reset();
        name = "init0";
    }

    public MethodCalls(short a) {
        reset();
        name = "init1";
        MethodCalls.a = a;

    }

    public MethodCalls(short a, Object b) {
        reset();
        name = "init2";
        MethodCalls.a = a;
        MethodCalls.b = b;
    }

    public MethodCalls(short a, Object b, double c) {
        reset();
        name = "init3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
    }

    public void dynVoidMethod0() {
        reset();
        name = "dynVoidMethod0";
    }

    public void dynVoidMethod1(short a) {
        reset();
        name = "dynVoidMethod1";
        MethodCalls.a = a;
    }

    public void dynVoidMethod2(short a, Object b) {
        reset();
        name = "dynVoidMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
    }

    public void dynVoidMethod3(short a, Object b, double c) {
        reset();
        name = "dynVoidMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;

        System.err.println("dynVoidMethod3 C=" + c);
    }

    public Object dynObjectMethod0() {
        reset();
        name = "dynObjectMethod0";
        return new Object();
    }

    public Object dynObjectMethod1(short a) {
        reset();
        name = "dynObjectMethod1";
        MethodCalls.a = a;
        return new Object();
    }

    public Object dynObjectMethod2(short a, Object b) {
        reset();
        name = "dynObjectMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return new Object();
    }

    public Object dynObjectMethod3(short a, Object b, double c) {
        reset();
        name = "dynObjectMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return new Object();
    }

    public boolean dynBooleanMethod0() {
        reset();
        name = "dynBooleanMethod0";
        return true;
    }

    public boolean dynBooleanMethod1(short a) {
        reset();
        name = "dynBooleanMethod1";
        MethodCalls.a = a;
        return true;
    }

    public boolean dynBooleanMethod2(short a, Object b) {
        reset();
        name = "dynBooleanMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return true;
    }

    public boolean dynBooleanMethod3(short a, Object b, double c) {
        reset();
        name = "dynBooleanMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return true;
    }

    public byte dynByteMethod0() {
        reset();
        name = "dynByteMethod0";
        return 1;
    }

    public byte dynByteMethod1(short a) {
        reset();
        name = "dynByteMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public byte dynByteMethod2(short a, Object b) {
        reset();
        name = "dynByteMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public byte dynByteMethod3(short a, Object b, double c) {
        reset();
        name = "dynByteMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public char dynCharMethod0() {
        reset();
        name = "dynCharMethod0";
        return 1;
    }

    public char dynCharMethod1(short a) {
        reset();
        name = "dynCharMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public char dynCharMethod2(short a, Object b) {
        reset();
        name = "dynCharMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public char dynCharMethod3(short a, Object b, double c) {
        reset();
        name = "dynCharMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public short dynShortMethod0() {
        reset();
        name = "dynShortMethod0";
        return 1;
    }

    public short dynShortMethod1(short a) {
        reset();
        name = "dynShortMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public short dynShortMethod2(short a, Object b) {
        reset();
        name = "dynShortMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public short dynShortMethod3(short a, Object b, double c) {
        reset();
        name = "dynShortMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        System.err.println("dynShortMethod3 C=" + c);
        return 1;
    }

    public int dynIntMethod0() {
        reset();
        name = "dynIntMethod0";
        return 1;
    }

    public int dynIntMethod1(short a) {
        reset();
        name = "dynIntMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public int dynIntMethod2(short a, Object b) {
        reset();
        name = "dynIntMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public int dynIntMethod3(short a, Object b, double c) {
        reset();
        name = "dynIntMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public long dynLongMethod0() {
        reset();
        name = "dynLongMethod0";
        return 1;
    }

    public long dynLongMethod1(short a) {
        reset();
        name = "dynLongMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public long dynLongMethod2(short a, Object b) {
        reset();
        name = "dynLongMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public long dynLongMethod3(short a, Object b, double c) {
        reset();
        name = "dynLongMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public float dynFloatMethod0() {
        reset();
        name = "dynFloatMethod0";
        return 1;
    }

    public float dynFloatMethod1(short a) {
        reset();
        name = "dynFloatMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public float dynFloatMethod2(short a, Object b) {
        reset();
        name = "dynFloatMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public float dynFloatMethod3(short a, Object b, double c) {
        reset();
        name = "dynFloatMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
    }

    public double dynDoubleMethod0() {
        reset();
        name = "dynDoubleMethod0";
        return 1;
    }

    public double dynDoubleMethod1(short a) {
        reset();
        name = "dynDoubleMethod1";
        MethodCalls.a = a;
        return 1;
    }

    public double dynDoubleMethod2(short a, Object b) {
        reset();
        name = "dynDoubleMethod2";
        MethodCalls.a = a;
        MethodCalls.b = b;
        return 1;
    }

    public double dynDoubleMethod3(short a, Object b, double c) {
        reset();
        name = "dynDoubleMethod3";
        MethodCalls.a = a;
        MethodCalls.b = b;
        MethodCalls.c = c;
        return 1;
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