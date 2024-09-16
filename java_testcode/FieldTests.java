import java.lang.System;

public class FieldTests {

    public static boolean staticBool = false;
    public static byte staticByte = 0;
    public static short staticShort = 0;
    public static char staticChar = 0;
    public static int staticInt = 0;
    public static long staticLong = 0;
    public static float staticFloat = 0f;
    public static double staticDouble = 0d;
    public static java.lang.Object staticObject = null;
    
    public static final FieldTests staticInstance = new FieldTests();

    public boolean dynBool = false;
    public byte dynByte = 0;
    public short dynShort = 0;
    public char dynChar = 0;
    public int dynInt = 0;
    public long dynLong = 0;
    public float dynFloat = 0f;
    public double dynDouble = 0d;
    public java.lang.Object dynObject = null;

    public static void dump() {
        System.out.println("staticFloat=" + staticFloat);
        System.out.println("staticDouble=" + staticDouble);
        System.out.println("dynFloat=" + staticInstance.dynFloat);
        System.out.println("dynDouble=" + staticInstance.dynDouble);
    }

    public static strictfp void add() {
        staticBool = !staticBool;
        staticByte = (byte) (staticByte + 1);
        staticShort = (short) (staticShort + 1);
        staticChar = (char) (staticChar + 1);
        staticInt = (int) (staticInt + 1);
        staticLong = (long) (staticLong + 1);
        staticFloat = (float) (staticFloat + 1f);
        staticDouble = (double) (staticDouble + 1d);
        staticInstance.dynBool = !staticInstance.dynBool;
        staticInstance.dynByte = (byte) (staticInstance.dynByte + 1);
        staticInstance.dynShort = (short) (staticInstance.dynShort + 1);
        staticInstance.dynChar = (char) (staticInstance.dynChar + 1);
        staticInstance.dynInt = (int) (staticInstance.dynInt + 1);
        staticInstance.dynLong = (long) (staticInstance.dynLong + 1L);
        staticInstance.dynFloat = (float) (staticInstance.dynFloat + 1f);
        staticInstance.dynDouble = (double) (staticInstance.dynDouble + 1d);

    }

    public static void reset() {
        staticBool = false;
        staticByte = 0;
        staticShort = 0;
        staticChar = 0;
        staticInt = 0;
        staticLong = 0;
        staticFloat = 0f;
        staticDouble = 0d;
        staticObject = null;
        staticInstance.dynBool = false;
        staticInstance.dynByte = 0;
        staticInstance.dynShort = 0;
        staticInstance.dynChar = 0;
        staticInstance.dynInt = 0;
        staticInstance.dynLong = 0;
        staticInstance.dynFloat = 0f;
        staticInstance.dynDouble = 0d;
        staticInstance.dynObject = null;
    }
}