// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension PerpetualType {
    var baseAsset: Asset {
        switch self {
        case let .open(data), let .close(data), let .increase(data): data.baseAsset
        case let .modify(data): data.baseAsset
        case let .reduce(reduceData): reduceData.data.baseAsset
        }
    }

    var data: PerpetualConfirmData? {
        switch self {
        case let .open(data), let .close(data), let .increase(data): data
        case let .reduce(reduceData): reduceData.data
        case .modify: nil
        }
    }
}
