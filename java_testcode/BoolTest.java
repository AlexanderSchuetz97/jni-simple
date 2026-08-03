public class BoolTest {
    public static boolean[] ARRAY = new boolean[2];

    public static boolean bool1() {
        return ARRAY[0];
    }

    public static boolean bool2() {
        return ARRAY[1];
    }

    public static int testIt() {
        if (ARRAY[0]) {
            if (ARRAY[1]) {
                return 2;
            }
            return 1;
        }

        if (ARRAY[1]) {
            return 3;
        }

        return 0;
    }
}