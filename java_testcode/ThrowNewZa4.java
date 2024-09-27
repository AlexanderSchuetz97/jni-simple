public abstract class ThrowNewZa4 extends Throwable {

    public static String message;

    public ThrowNewZa4(String arg) {
        super(arg);
        message = String.valueOf(arg);
    }

    public static boolean isAbstractInst(Object obj) {
        return obj.getClass() == ThrowNewZa4.class;
    }
}