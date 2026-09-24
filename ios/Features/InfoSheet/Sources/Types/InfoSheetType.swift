// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import struct Gemstone.GemConfirmErrorInfo
import enum Gemstone.GemInfoTopic
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public enum InfoSheetType: Identifiable, Sendable, Equatable {
    case networkFee(Asset)
    case balanceRequired(GemConfirmErrorInfo, image: AssetImage, button: InfoSheetButton?)
    case insufficientNetworkFee(GemConfirmErrorInfo, image: AssetImage, button: InfoSheetButton?)
    case transactionState(imageURL: URL?, placeholder: Image?, model: TransactionStateViewModel)
    case estimatedConfirmation(Chain)
    case watchWallet
    case paymentVerification
    case stakeLockTime(Image?)
    case stakeApr(Image?)
    case dustThreshold(Chain, image: AssetImage)
    // swaps
    case priceImpact
    case slippage
    case noQuote
    // asset
    case assetStatus(VerificationStatus)
    case accountMinimalBalance(GemConfirmErrorInfo)
    /// stake / perpetual / earn
    case minimumAmount(Asset, required: BigInt, action: InfoSheetAction?)
    case swapMinimumAmount(GemConfirmErrorInfo, providerName: String, image: AssetImage, button: InfoSheetButton?)
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
        case let .insufficientNetworkFee(info, _, _): "insufficientNetworkFee_\(info.title)"
        case let .balanceRequired(info, _, _): "balanceRequired_\(info.title)"
        case let .transactionState(_, _, model): model.state.id
        case let .estimatedConfirmation(chain): "estimatedConfirmation_\(chain.rawValue)"
        case .watchWallet: "watchWallet"
        case .paymentVerification: "paymentVerification"
        case .stakeLockTime: "stakeLockTime"
        case .stakeApr: "stakeApr"
        case .priceImpact: "priceImpact"
        case .slippage: "slippage"
        case let .assetStatus(status): "assetStatus_\(status.rawValue)"
        case let .accountMinimalBalance(info): "accountMinimalBalance_\(info.title)"
        case let .minimumAmount(asset, amount, _): "minimumAmount_\(asset.id.identifier)\(amount)"
        case let .swapMinimumAmount(info, providerName, _, _): "swapMinimumAmount_\(info.title)\(providerName)"
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

public extension InfoSheetType {
    init(topic: GemInfoTopic, assetImage: AssetImage?, buyAction: InfoSheetAction? = nil) {
        self = switch topic {
        case let .networkFee(asset): .networkFee(asset.toPrimitives())
        case let .minimumAmount(asset, minimum): .minimumAmount(asset.toPrimitives(), required: minimum, action: buyAction)
        case .noQuote: .noQuote
        case .priceImpact: .priceImpact
        case .slippage: .slippage
        case .openInterest: .openInterest
        case .fundingApr: .fundingApr
        case .stakeApr: .stakeApr(assetImage?.placeholder)
        case .stakeLockTime: .stakeLockTime(assetImage?.placeholder)
        case .stakeFrozenRequired: .stakeFrozenRequired
        case .autoClose: .autoclose
        case .liquidationPrice: .liquidationPrice
        case .fundingPayments: .fundingPayments
        case .fullyDilutedValuation: .fullyDilutedValuation
        case .circulatingSupply: .circulatingSupply
        case .totalSupply: .totalSupply
        case .maxSupply: .maxSupply
        case let .transactionStatus(state, tone):
            .transactionState(
                imageURL: assetImage?.imageURL,
                placeholder: assetImage?.placeholder,
                model: TransactionStateViewModel(state: state.toPrimitives(), tone: tone),
            )
        }
    }
}
