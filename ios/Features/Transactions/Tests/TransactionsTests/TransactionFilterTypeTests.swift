// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing
import Transactions

struct TransactionFilterTypeTests {
    @Test
    func map() {
        for type in TransactionType.allCases {
            switch type {
            case .transfer:
                #expect(type.filterType == .transfers)
            case .transferNFT:
                #expect(type.filterType == .transfers)
            case .smartContractCall:
                #expect(type.filterType == .smartContract)
            case .swap:
                #expect(type.filterType == .swaps)
            case .tokenApproval:
                #expect(type.filterType == .swaps)
            case .stakeDelegate, .stakeUndelegate, .stakeRewards, .stakeRedelegate, .stakeWithdraw, .stakeFreeze, .stakeUnfreeze:
                #expect(type.filterType == .stake)
            case .assetActivation:
                #expect(type.filterType == .others)
            case .perpetualOpenPosition, .perpetualClosePosition, .perpetualModifyPosition:
                #expect(type.filterType == .perpetuals)
            case .earnDeposit, .earnWithdraw:
                #expect(type.filterType == .stake)
            }
        }
    }

    @Test
    func everyTransactionTypeIsSelectableByItsOwnGroup() {
        var model = TransactionTypesFilterViewModel(types: TransactionType.allCases)

        for type in TransactionType.allCases {
            model.selectedTypes = [type.filterType]
            #expect(model.requestFilters.contains(type))
        }
    }
}
