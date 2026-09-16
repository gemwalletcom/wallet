// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemBalanceResource
import func Gemstone.balanceResourceRows
import BigInt
import Formatters
import Foundation
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct BalanceViewModel: Sendable {
    private static let fullFormatter = ValueFormatter(style: .full)

    private let asset: Asset
    private let balance: Balance
    private let formatter: ValueFormatter

    public init(
        asset: Asset,
        balance: Balance,
        formatter: ValueFormatter,
    ) {
        self.asset = asset
        self.balance = balance
        self.formatter = formatter
    }

    public var balanceAmount: Double {
        do {
            return try Self.fullFormatter.double(from: total, decimals: asset.decimals.asInt)
        } catch {
            return .zero
        }
    }

    public var availableBalanceAmount: Double {
        do {
            return try Self.fullFormatter.double(from: balance.available, decimals: asset.decimals.asInt)
        } catch {
            return .zero
        }
    }

    public var balanceText: String {
        guard !total.isZero else {
            return .zero
        }
        return formatter.string(total, decimals: asset.decimals.asInt)
    }

    public var availableBalanceText: String {
        guard !balance.available.isZero else {
            return .zero
        }
        return formatter.string(balance.available, decimals: asset.decimals.asInt)
    }

    public var totalBalanceTextWithSymbol: String {
        formatter.string(total, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var availableBalanceTextWithSymbol: String {
        formatter.string(balance.available, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public func balanceTextWithSymbol(_ value: BigInt) -> String {
        formatter.string(value, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var balanceTextColor: Color {
        guard !total.isZero else {
            return Colors.gray
        }
        return Colors.black
    }

    public var energyText: String {
        resourceText(.energy)
    }

    public var bandwidthText: String {
        resourceText(.bandwidth)
    }

    private func resourceText(_ resource: GemBalanceResource) -> String {
        balanceResourceRows(metadata: balance.metadata?.toGem())
            .first { $0.resource == resource }?
            .text ?? ""
    }

    var total: BigInt {
        balance.total
    }
}
