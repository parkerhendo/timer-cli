import Foundation
import Combine

@MainActor
final class TimerState: ObservableObject {
    @Published var currentFrame: Frame?
    @Published var recentProjects: [String] = []
    @Published var lastError: String?

    private var refreshTimer: Timer?
    private var fileMonitor: DispatchSourceFileSystemObject?

    var isTracking: Bool { currentFrame != nil }

    init() {
        refresh()
        startPolling()
        watchDatabase()
    }

    deinit {
        // Timer.invalidate() and DispatchSource.cancel() are thread-safe
        // Capture values to avoid accessing self after deallocation starts
        let timer = refreshTimer
        let monitor = fileMonitor
        timer?.invalidate()
        monitor?.cancel()
    }

    func refresh() {
        currentFrame = TimerDatabase.shared?.currentFrame()
        recentProjects = TimerDatabase.shared?.recentProjects() ?? []
    }

    func start(project: String) {
        do {
            try TimerDatabase.shared?.start(project: project)
            lastError = nil
            refresh()
        } catch {
            lastError = "Failed to start: \(error.localizedDescription)"
        }
    }

    func stop() {
        do {
            try TimerDatabase.shared?.stop()
            lastError = nil
            refresh()
        } catch {
            lastError = "Failed to stop: \(error.localizedDescription)"
        }
    }

    private func startPolling() {
        refreshTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.refresh()
            }
        }
    }

    private func watchDatabase() {
        guard let path = ProcessInfo.processInfo.environment["TIMER_CLI_DB"] ??
              FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first?
                .appendingPathComponent("timer-cli/frames.db").path else { return }

        let fd = open(path, O_EVTONLY)
        guard fd >= 0 else { return }

        let monitor = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .extend],
            queue: .main
        )

        fileMonitor = monitor

        fileMonitor?.setEventHandler { [weak self] in
            self?.refresh()
        }

        fileMonitor?.setCancelHandler { [fd] in
            close(fd)
        }

        fileMonitor?.resume()
    }
}
