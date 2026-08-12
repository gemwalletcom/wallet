// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public enum InfoSheetType: Identifiable, Sendable, Equatable {
    case networkFee(Chain)
    case balanceRequired(Asset, image: AssetImage, requirement: BalanceRequirement, action: InfoSheetAction)
    case insufficientNetworkFee(Asset, image: AssetImage, requirement: BalanceRequirement?, price: Price?, currency: String, action: InfoSheetAction)
    case transactionState(imageURL: URL?, placeholder: Image?, state: TransactionState)
    case estimatedConfirmation(Chain)
    case watchWallet
    case stakeLockTime(Image?)
    case stakeApr(Image?)
    case dustThreshold(Chain, image: AssetImage)
    // swaps
    case priceImpact
    case slippage
    case noQuote
    // asset
    case assetStatus(AssetScoreType)
    case accountMinimalBalance(Asset, required: BigInt)
    /// stake / perpetual / earn
    case minimumAmount(Asset, required: BigInt, action: InfoSheetAction)
    // stake
    case stakingReservedFees(image: AssetImage)
    case pendingUnconfirmedBalance
    case stakeFrozenRequired
    // perpetuals
    case fundingApr
    case fundingPayments
    case liquidationPrice
    case openInterest
    case autoclose
    // scan transaction
    case maliciousTransaction
    case memoRequired(symbol: String)
    // market
    case fullyDilutedValuation
    case circulatingSupply
    case totalSupply
    case maxSupply

    public var id: String {
        switch self {
        case .networkFee: "networkFees"
        case let .insufficientNetworkFee(asset, _, _, _, _, _): "insufficientNetworkFee_\(asset.id.identifier)"
        case let .balanceRequired(asset, _, _, _): "balanceRequired_\(asset.id.identifier)"
        case let .transactionState(_, _, state): state.id
        case let .estimatedConfirmation(chain): "estimatedConfirmation_\(chain.rawValue)"
        case .watchWallet: "watchWallet"
        case .stakeLockTime: "stakeLockTime"
        case .stakeApr: "stakeApr"
        case .priceImpact: "priceImpact"
        case .slippage: "slippage"
        case let .assetStatus(status): "assetStatus_\(status.rawValue)"
        case let .accountMinimalBalance(asset, amount): "accountMinimalBalance_\(asset.id.identifier)\(amount)"
        case let .minimumAmount(asset, amount, _): "minimumAmount_\(asset.id.identifier)\(amount)"
        case .stakingReservedFees: "stakingReservedFees"
        case .pendingUnconfirmedBalance: "pendingUnconfirmedBalance"
        case .stakeFrozenRequired: "stakeFrozenRequired"
        case .noQuote: "noQuote"
        case .fundingApr: "fundingApr"
        case .fundingPayments: "fundingPayments"
        case .liquidationPrice: "liquidationPrice"
        case .openInterest: "openInterest"
        case .autoclose: "autoClose"
        case .maliciousTransaction: "maliciousTransaction"
        case let .memoRequired(symbol): "memoRequired_\(symbol)"
        case let .dustThreshold(chain, _): "dustThreshold_\(chain.rawValue)"
        case .fullyDilutedValuation: "fullyDilutedValuation"
        case .circulatingSupply: "circulatingSupply"
        case .totalSupply: "totalSupply"
        case .maxSupply: "maxSupply"
        }
    }

    public static func == (lhs: InfoSheetType, rhs: InfoSheetType) -> Bool {
        lhs.id == rhs.id
    }
}
