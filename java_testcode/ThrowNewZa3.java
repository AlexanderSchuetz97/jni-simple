public class ThrowNewZa3 extends Throwable {

    public static String message;

    public ThrowNewZa3(Object arg) {
        super();
        message = String.valueOf(arg);
    }

}