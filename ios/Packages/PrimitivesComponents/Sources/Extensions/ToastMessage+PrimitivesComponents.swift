// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style

public extension ToastMessage {
    static func transfer(for type: TransferDataType) -> ToastMessage? {
        guard case let .perpetual(_, perpetualType) = type else {
            return nil
        }
        return switch perpetualType {
        case let .open(data): .success(Localized.Perpetual.openDirection(PerpetualDirectionViewModel(direction: data.direction).title))
        case .close: .success(Localized.Perpetual.closePosition)
        case .modify: .success(Localized.Perpetual.modifyPosition)
        case .increase: .success(Localized.Perpetual.increasePosition)
        case .reduce: .success(Localized.Perpetual.reducePosition)
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
