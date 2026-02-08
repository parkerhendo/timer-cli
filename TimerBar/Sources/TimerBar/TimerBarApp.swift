import SwiftUI

@main
struct TimerBarApp: App {
    @StateObject private var timer = TimerState()

    var body: some Scene {
        MenuBarExtra {
            MenuView(timer: timer)
        } label: {
            if let frame = timer.currentFrame {
                Text("\(frame.project) - \(frame.formattedDuration)")
            } else {
                Text("--:--")
            }
        }
    }
}
