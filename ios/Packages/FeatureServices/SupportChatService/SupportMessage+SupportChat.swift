// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import UniformTypeIdentifiers

extension SupportMessage {
    static func userText(_ content: String) -> SupportMessage {
        SupportMessage(
            id: UUID().uuidString,
            content: content,
            sender: .user,
            status: .sending,
            createdAt: .now,
            images: [],
        )
    }

    static func userImage(id: String, url: URL, fileName: String, fileSize: Int) -> SupportMessage {
        SupportMessage(
            id: id,
            content: "",
            sender: .user,
            status: .sending,
            createdAt: .now,
            images: [SupportMessageImage(
                id: id,
                url: url.absoluteString,
                thumbnailUrl: nil,
                fileName: fileName,
                fileSize: UInt64(fileSize),
                width: nil,
                height: nil,
            )],
        )
    }

    func with(status: SupportMessageDeliveryStatus) -> SupportMessage {
        SupportMessage(
            id: id,
            content: content,
            sender: sender,
            status: status,
            createdAt: createdAt,
            images: images,
        )
    }

    func with(images: [SupportMessageImage]) -> SupportMessage {
        SupportMessage(
            id: id,
            content: content,
            sender: sender,
            status: status,
            createdAt: createdAt,
            images: images,
        )
    }
}

extension SupportMessageImage {
    var fileExtension: String {
        let name = fileName ?? url.asURL?.lastPathComponent
        if let pathExtension = name.map({ ($0 as NSString).pathExtension }), pathExtension.isNotEmpty {
            return pathExtension.lowercased()
        }
        return UTType(mimeType: mimeType)?.preferredFilenameExtension ?? "jpg"
    }

    var mimeType: String {
        let fileExtension = fileName.map { ($0 as NSString).pathExtension } ?? ""
        return UTType(filenameExtension: fileExtension)?.preferredMIMEType ?? "image/jpeg"
    }

    func with(url: String) -> SupportMessageImage {
        SupportMessageImage(id: id, url: url, thumbnailUrl: thumbnailUrl, fileName: fileName, fileSize: fileSize, width: width, height: height)
    }
}
