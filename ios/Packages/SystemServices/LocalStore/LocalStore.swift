// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct LocalStore: Sendable {
    private var fileManager: FileManager {
        FileManager.default
    }

    private let documentDirectory = URL.documentsDirectory

    public init() {}

    // MARK: - Public methods

    public func store(_ data: Data, id: String, documentType: String) throws -> String {
        let documentPath = documentPath(for: id, documentType: documentType)
        try data.write(to: documentPath, options: .atomic)
        return documentPath.lastPathComponent
    }

    public func remove(_ fileName: String) throws {
        let url = documentDirectory.appendingPathComponent(fileName)
        guard fileManager.fileExists(atPath: url.path()) else {
            return
        }
        try fileManager.removeItem(at: url)
    }

    // MARK: - Private methods

    private func documentPath(for id: String, documentType: String?) -> URL {
        let path = documentDirectory.appendingPathComponent(id)
        if path.pathExtension.isEmpty, let documentType {
            return path.appendingPathExtension(documentType)
        }
        return path
    }
}
