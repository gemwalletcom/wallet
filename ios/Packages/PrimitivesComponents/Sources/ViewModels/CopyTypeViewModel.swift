// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Localization
import Primitives
import Style
import UIKit

public struct CopyTypeViewModel: Equatable, Hashable, Sendable {
    public let type: CopyType
    public let copyValue: String

    public init(type: CopyType, copyValue: String) {
        self.type = type
        self.copyValue = copyValue
    }

    public var message: String {
        switch type {
        case .secretPhrase: Localized.Common.copied(Localized.Common.secretPhrase)
        case .privateKey: Localized.Common.copied(Localized.Common.privateKey)
        case let .address(asset, address):
            Localized.Common.copied(
                String(
                    format: "%@ (%@) ",
                    asset.name,
                    AddressFormatter(style: .short, address: address, chain: asset.chain).value(),
                ),
            )
        }
    }

    public var systemImage: String {
        SystemImage.copy
    }

    public var expirationTimeInternal: TimeInterval? {
        switch type {
        case .secretPhrase, .privateKey: 60
        case .address: .none
        }
    }

    public func copy() {
        Self.copyToClipboard(copyValue, expirationTime: expirationTimeInternal)
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
