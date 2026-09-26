// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemCopy
import GemstonePrimitives
import Localization
import Primitives
import Style
import UIKit

public struct CopyTypeViewModel: Equatable, Hashable, Sendable {
    public let content: GemCopy

    public init(content: GemCopy) {
        self.content = content
    }

    public var message: String {
        content.kind.copiedMessage(display: content.display)
    }

    public var systemImage: String {
        SystemImage.copy
    }

    public var expirationTimeInternal: TimeInterval? {
        content.kind.clipboardExpirySeconds().map(TimeInterval.init)
    }

    public func copy() {
        Self.copyToClipboard(content.value, expirationTime: expirationTimeInternal)
    }

    public static func copyToClipboard(_ value: String, expirationTime: TimeInterval?) {
        let options = pasteboardOptions(expirationTime: expirationTime)
        UIPasteboard.general.setItems([[UIPasteboard.typeAutomatic: value]], options: options)
    }

    public static func clearClipboard() {
        UIPasteboard.general.items = []
    }

    static func pasteboardOptions(expirationTime: TimeInterval?) -> [UIPasteboard.OptionsKey: Any] {
        var options: [UIPasteboard.OptionsKey: Any] = [:]

        if let expirationTime {
            options[.localOnly] = true
            options[.expirationDate] = Date().addingTimeInterval(expirationTime)
        }

        return options
    }
}
