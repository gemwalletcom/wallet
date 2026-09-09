// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemTransactionDetailRow
import Foundation
import Primitives
import PrimitivesComponents
import SwiftUI

public enum TransactionSectionType: String, Identifiable, Equatable {
    case header
    case swapProgress
    case swapAction
    case details
    case fee
    case explorer

    public var id: String {
        rawValue
    }

    init(first item: TransactionItem) {
        self = switch item {
        case .header: .header
        case .swapProgress: .swapProgress
        case .swapButton: .swapAction
        case .fee: .fee
        case .explorerLink: .explorer
        case .date, .status, .estimatedConfirmation, .participant, .memo, .resource, .rate, .network, .pnl, .price, .provider: .details
        }
    }
}

public enum TransactionItem: Identifiable, Equatable, Sendable {
    case header
    case swapProgress
    case swapButton
    case date
    case status
    case estimatedConfirmation
    case participant
    case memo
    case resource
    case rate
    case network
    case pnl
    case price
    case provider
    case fee
    case explorerLink

    public var id: Self {
        self
    }

    init(_ row: GemTransactionDetailRow) {
        self = switch row {
        case .header: .header
        case .swapProgress: .swapProgress
        case .swapAgain: .swapButton
        case .date: .date
        case .status: .status
        case .estimatedConfirmation: .estimatedConfirmation
        case .participant: .participant
        case .memo: .memo
        case .resource: .resource
        case .rate: .rate
        case .network: .network
        case .provider: .provider
        case .pnl: .pnl
        case .price: .price
        case .fee: .fee
        case .explorer: .explorerLink
        }
    }
}

public enum TransactionItemModel {
    case listItem(ListItemModel)
    case fee(ListItemModel)
    case header(TransactionHeaderItemModel)
    case swapProgress(TransactionSwapProgressItemModel)
    case participant(TransactionParticipantItemModel)
    case rate(title: String, value: String)
    case network(title: String, subtitle: String, image: AssetImage)
    case pnl(title: String, value: String, color: Color)
    case price(title: String, value: String)
    case explorer(url: URL, text: String)
    case swapAgain(text: String)
    case empty
}

public extension ListSection where T == TransactionItem {
    init(type: TransactionSectionType, _ items: [TransactionItem]) {
        self.init(type: type, values: items)
    }
}
