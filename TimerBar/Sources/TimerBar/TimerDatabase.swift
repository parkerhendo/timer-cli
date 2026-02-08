import Foundation
import SQLite

final class TimerDatabase {
    private let db: Connection

    static let shared: TimerDatabase? = {
        do {
            return try TimerDatabase()
        } catch {
            print("Failed to open database: \(error)")
            return nil
        }
    }()

    private init() throws {
        let path = try Self.databasePath()
        db = try Connection(path)
    }

    private static func databasePath() throws -> String {
        if let envPath = ProcessInfo.processInfo.environment["TIMER_CLI_DB"] {
            return envPath
        }
        guard let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            throw NSError(domain: "TimerDatabase", code: 1, userInfo: [NSLocalizedDescriptionKey: "Could not find Application Support directory"])
        }
        let dataDir = appSupport.appendingPathComponent("timer-cli")
        try FileManager.default.createDirectory(at: dataDir, withIntermediateDirectories: true)
        return dataDir.appendingPathComponent("frames.db").path
    }

    func currentFrame() -> Frame? {
        let query = """
            SELECT id, project, start_time, end_time, tags
            FROM frames WHERE end_time IS NULL LIMIT 1
        """
        do {
            for row in try db.prepare(query) {
                return parseFrame(row)
            }
        } catch {
            print("Query error: \(error)")
        }
        return nil
    }

    func recentProjects(limit: Int = 10) -> [String] {
        let query = """
            SELECT project FROM frames
            GROUP BY project
            ORDER BY MAX(start_time) DESC LIMIT ?
        """
        var projects: [String] = []
        do {
            for row in try db.prepare(query, limit) {
                if let project = row[0] as? String {
                    projects.append(project)
                }
            }
        } catch {
            print("Query error: \(error)")
        }
        return projects
    }

    func start(project: String, tags: [String] = []) throws {
        let tagsStr = tags.isEmpty ? nil : tags.joined(separator: ",")
        let now = Int64(Date().timeIntervalSince1970)
        try db.run(
            "INSERT INTO frames (project, start_time, tags) VALUES (?, ?, ?)",
            project, now, tagsStr
        )
    }

    func stop() throws {
        let now = Int64(Date().timeIntervalSince1970)
        try db.run("UPDATE frames SET end_time = ? WHERE end_time IS NULL", now)
    }

    private func parseFrame(_ row: Statement.Element) -> Frame? {
        guard let id = row[0] as? Int64,
              let project = row[1] as? String,
              let startTs = row[2] as? Int64 else {
            return nil
        }
        let endTs = row[3] as? Int64
        let tagsStr = row[4] as? String

        return Frame(
            id: id,
            project: project,
            startTime: Date(timeIntervalSince1970: Double(startTs)),
            endTime: endTs.map { Date(timeIntervalSince1970: Double($0)) },
            tags: tagsStr?.split(separator: ",").map(String.init) ?? []
        )
    }
}
