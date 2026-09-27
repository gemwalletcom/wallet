// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemCopy
import UIKit

public enum Clipboard {
    public static func copy(_ copy: GemCopy) {
        self.copy(copy.value, expirationTime: expirationTime(copy))
    }

    public static func copy(_ value: String, expirationTime: TimeInterval?) {
        UIPasteboard.general.setItems([[UIPasteboard.typeAutomatic: value]], options: pasteboardOptions(expirationTime: expirationTime))
    }

    public static func clear() {
        UIPasteboard.general.items = []
    }

    static func expirationTime(_ copy: GemCopy) -> TimeInterval? {
        copy.kind.clipboardExpirySeconds().map(TimeInterval.init)
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
