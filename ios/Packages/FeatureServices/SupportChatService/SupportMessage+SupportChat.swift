// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

extension SupportMessage {
    static func pending(_ content: SupportMessageContent) -> SupportMessage {
        let id = UUID().uuidString
        return SupportMessage(
            id: id,
            content: content.text,
            sender: .user,
            status: .sending,
            createdAt: .now,
            images: content.images(id: id),
        )
    }

    func with(status: SupportMessageStatus) -> SupportMessage {
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

private extension SupportMessageContent {
    var text: String {
        switch self {
        case let .text(text): text
        case .image: ""
        }
    }

    func images(id: String) -> [SupportMessageImage] {
        switch self {
        case .text: []
        case let .image(attachment):
            [SupportMessageImage(
                id: id,
                url: "",
                thumbnailUrl: nil,
                fileName: attachment.fileName,
                fileSize: UInt64(attachment.data.count),
                width: nil,
                height: nil,
            )]
        }
    }
}
