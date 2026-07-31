/**
 * Java side of the jboolean reproducer.
 *
 * Supplies a boolean the probe can poke a raw 0x02 into, and booleans that are
 * genuinely false. Everything interesting is in rust/probe/src/lib.rs.
 */
public class JBoolMre {

    /** [1] The probe writes a raw 0x02 into element 0 with SetBooleanArrayRegion. */
    public boolean[] arr = new boolean[1];

    /** Hands that byte back out through CallBooleanMethodA. */
    public boolean readArr() {
        return arr[0];
    }

    /** [2] Genuinely false. */
    public boolean returnFalse() {
        return false;
    }

    /** [2] Never assigned, so also false. */
    public boolean flagFalse;

    /** Implemented by whichever build of the probe was loaded. */
    private static native String rustProbe();

    public static void main(String[] args) {
        String lib = System.getProperty("jbool.lib");
        if (lib == null) {
            System.out.println("ERROR|jbool.lib property not set");
            System.exit(2);
        }
        System.load(lib);
        System.out.println(rustProbe());
    }
}
