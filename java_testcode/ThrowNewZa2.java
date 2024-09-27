public class ThrowNewZa2 extends Throwable {

    public static String message;

    public ThrowNewZa2() {
        super();
        message = "called";
    }

        public ThrowNewZa2(String arg) {
            super();
            message = String.valueOf(arg);
        }

}