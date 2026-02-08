import Foundation

struct Frame: Identifiable {
    let id: Int64
    let project: String
    let startTime: Date
    let endTime: Date?
    let tags: [String]

    var isActive: Bool { endTime == nil }

    var duration: TimeInterval {
        let end = endTime ?? Date()
        return end.timeIntervalSince(startTime)
    }

    var formattedDuration: String {
        let total = Int(duration)
        let hours = total / 3600
        let minutes = (total % 3600) / 60
        let seconds = total % 60

        if hours > 0 {
            return String(format: "%d:%02d:%02d", hours, minutes, seconds)
        } else {
            return String(format: "%d:%02d", minutes, seconds)
        }
    }

    var tagsDisplay: String {
        tags.map { "+\($0)" }.joined(separator: " ")
    }
}
