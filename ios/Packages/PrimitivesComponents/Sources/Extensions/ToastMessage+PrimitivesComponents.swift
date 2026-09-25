// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemSubmitMessage
import enum Gemstone.GemSubmitResult
import GemstonePrimitives
import Localization
import Primitives
import Style

public extension ToastMessage {
    static func transfer(_ result: GemSubmitResult) -> ToastMessage? {
        let message: GemSubmitMessage? = switch result {
        case let .sent(_, message), let .signed(_, message): message
        }
        return switch message {
        case let .warning(text): .error(text.text)
        case let .confirmed(text): .success(text.text)
        case nil: nil
        }
    }

    static func copied(_ value: String) -> ToastMessage {
        ToastMessage(title: Localized.Common.copied(value), image: SystemImage.copy)
    }

    static func copy(_ message: String) -> ToastMessage {
        ToastMessage(title: message, image: SystemImage.copy)
    }

    static func pin(_ name: String, pinned: Bool) -> ToastMessage {
        ToastMessage(
            title: pinned ? Localized.Common.pinnedAsset(name) : Localized.Common.unpinnedAsset(name),
            image: pinned ? SystemImage.pin : SystemImage.unpin,
        )
    }

    static func addedToWallet() -> ToastMessage {
        ToastMessage(title: Localized.Asset.addedToWallet, image: SystemImage.plusCircle)
    }

    static func showAsset(visible: Bool) -> ToastMessage {
        ToastMessage(
            title: visible ? Localized.Asset.addedToWallet : Localized.Asset.hiddenFromWallet,
            image: visible ? SystemImage.plusCircle : SystemImage.minusCircle,
        )
    }

    static func priceAlert(for assetName: String, enabled: Bool) -> ToastMessage {
        ToastMessage(
            title: enabled ? Localized.PriceAlerts.enabledFor(assetName) : Localized.PriceAlerts.disabledFor(assetName),
            image: SystemImage.bellFill,
        )
    }

    static func priceAlert(message: String) -> ToastMessage {
        ToastMessage(title: message, image: SystemImage.bellFill)
    }

    static func success(_ message: String) -> ToastMessage {
        ToastMessage(title: message, image: SystemImage.checkmark)
    }

    static func error(_ message: String) -> ToastMessage {
        ToastMessage(title: message, image: SystemImage.xmarkCircle)
    }
}
