import SwiftUI

struct MenuView: View {
    @ObservedObject var timer: TimerState

    var body: some View {
        if let frame = timer.currentFrame {
            trackingView(frame: frame)
        } else {
            idleView
        }

        Divider()

        Button("Quit") {
            NSApplication.shared.terminate(nil)
        }
        .keyboardShortcut("q")
    }

    private func trackingView(frame: Frame) -> some View {
        Group {
            Text(frame.project)
                .font(.headline)

            if !frame.tags.isEmpty {
                Text(frame.tagsDisplay)
                    .foregroundStyle(.secondary)
            }

            Text(frame.formattedDuration)
                .font(.system(.body, design: .monospaced))

            Divider()

            Button("Stop") {
                timer.stop()
            }
            .keyboardShortcut("s")
        }
    }

    private var idleView: some View {
        Group {
            Text("Not tracking")
                .foregroundStyle(.secondary)

            if !timer.recentProjects.isEmpty {
                Divider()

                Text("Recent")
                    .font(.caption)
                    .foregroundStyle(.tertiary)

                ForEach(timer.recentProjects.prefix(5), id: \.self) { project in
                    Button(project) {
                        timer.start(project: project)
                    }
                }
            }
        }
    }
}
