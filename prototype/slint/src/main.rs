slint::slint! {
    import { Colors } from "./src/ui/theme/colors.slint";

    export component PreviewWindow inherits Window {
        width: 400px;
        height: 200px;

        Rectangle {
            x: 10px;
            y: 10px;
            width: 90px;
            height: 90px;
            background: Colors.background;
        }

        Rectangle {
            x: 110px;
            y: 10px;
            width: 90px;
            height: 90px;
            background: Colors.surface;
        }

        Rectangle {
            x: 210px;
            y: 10px;
            width: 90px;
            height: 90px;
            background: Colors.primary;
        }

        Rectangle {
            x: 310px;
            y: 10px;
            width: 90px;
            height: 90px;
            background: Colors.text;
        }
    }
}

fn main() {
    let window = PreviewWindow::new().unwrap();
    window.show().unwrap();
}