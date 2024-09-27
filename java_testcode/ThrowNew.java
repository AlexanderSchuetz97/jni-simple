public class ThrowNew extends Throwable {

    public static String message;

    public ThrowNew(String msg) {
        super(msg);
        message = msg;
    }

}