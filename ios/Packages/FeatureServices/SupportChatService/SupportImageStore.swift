// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct SupportImageStore {
    enum Location {
        case uploads
        case previews
    }

    private let location: Location

    init(_ location: Location) {
        self.location = location
    }

    func store(_ data: Data, id: String, fileExtension: String) throws -> URL {
        let url = try fileURL(id: id, fileExtension: fileExtension)
        try data.write(to: url, options: .atomic)
        return url
    }

    func file(id: String, fileExtension: String) -> URL? {
        guard let url = try? fileURL(id: id, fileExtension: fileExtension), FileManager.default.fileExists(atPath: url.path) else {
            return nil
        }
        return url
    }

    func data(at url: URL) -> Data? {
        guard url.isFileURL else { return nil }
        return try? Data(contentsOf: url)
    }

    func remove(at url: URL) {
        guard url.isFileURL else { return }
        try? FileManager.default.removeItem(at: url)
    }
}

private extension SupportImageStore {
    func fileURL(id: String, fileExtension: String) throws -> URL {
        try directory().appendingPathComponent("\(id).\(fileExtension)")
    }

    func directory() throws -> URL {
        let base = try FileManager.default.url(for: location.searchPath, in: .userDomainMask, appropriateFor: nil, create: true)
        let directory = base.appendingPathComponent(location.directoryName, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        return directory
    }
}

private extension SupportImageStore.Location {
    var searchPath: FileManager.SearchPathDirectory {
        switch self {
        case .uploads: .applicationSupportDirectory
        case .previews: .cachesDirectory
        }
    }

    var directoryName: String {
        switch self {
        case .uploads: "support_uploads"
        case .previews: "support_image_previews"
        }
    }
}
