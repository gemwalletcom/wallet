// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public enum FeeOption: Sendable {
    case tokenAccountCreation
}

public typealias FeeOptionMap = [FeeOption: BigInt]

public struct Fee: Sendable {
    public let fee: BigInt
    public let gasPriceType: GasPriceType
    public let gasLimit: BigInt
    public let options: FeeOptionMap
    public let feeAssetId: AssetId

    public init(
        fee: BigInt,
        gasPriceType: GasPriceType,
        gasLimit: BigInt,
        options: FeeOptionMap = [:],
        feeAssetId: AssetId,
    ) {
        self.fee = fee
        self.gasPriceType = gasPriceType
        self.gasLimit = gasLimit
        self.options = options
        self.feeAssetId = feeAssetId
    }

    public var gasPrice: BigInt {
        gasPriceType.gasPrice
    }

}

// MARK: - Equatable

extension Fee: Equatable {}
