// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents

public enum ToastEventMessageFactory {
    public static func makeToastMessage(for event: ToastEvent) -> ToastMessage? {
        switch event {
        case let .transfer(data): transferMessage(for: data.type)
        }
    }
}

// MARK: - Private

extension ToastEventMessageFactory {
    private static func transferMessage(for type: TransferDataType) -> ToastMessage? {
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
}
