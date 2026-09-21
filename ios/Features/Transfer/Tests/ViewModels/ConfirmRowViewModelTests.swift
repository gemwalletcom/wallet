// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmDestination
import enum Gemstone.GemConfirmRowContent
import enum Gemstone.GemListRow
import GemstonePrimitives
import Localization
@testable import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmRowViewModelTests {
    @Test
    func sharedRowsPassThroughAndDetailsDrawNothing() {
        let memo = GemListRow.memo(value: "test memo", copy: "test memo")
        guard case let .row(row) = ConfirmRowViewModel(content: .row(row: memo)).itemModel else {
            Issue.record("Expected a shared row")
            return
        }
        #expect(row == memo)
        #expect(ConfirmRowViewModel(content: .details).isEmpty)
    }

    @Test
    func recipientFollowsTheDestination() throws {
        let cases: [(GemConfirmDestination, String, String)] = [
            (.recipient(name: nil, address: "0xrecipient"), Localized.Transfer.Recipient.title, "0xrecipient"),
            (.contract(address: "0xspender"), Localized.Asset.contract, "0xspender"),
            (.validator(name: "Allnodes", address: "validator1"), Localized.Stake.validator, "validator1"),
            (.provider(name: "Yo", address: "0xprovider"), Localized.Common.provider, "0xprovider"),
        ]
        for (destination, title, address) in cases {
            let item = try #require(model(destination).recipientItem)
            #expect(item.title == title)
            #expect(item.account.address == address)
        }
        #expect(try #require(model(.resource(resource: Resource.energy.toGem())).recipientItem).title == Localized.Stake.resource)
    }

    @Test
    func contactImage() throws {
        let withImage = try #require(model(.recipient(name: nil, address: "0x1"), addressName: .mock(type: .contact, imageUrl: "avatar.png")).recipientItem)
        let withoutImage = try #require(model(.recipient(name: nil, address: "0x1"), addressName: .mock(type: .contact, imageUrl: nil)).recipientItem)
        #expect(withImage.account.assetImage?.imageURL == ImageSource("avatar.png").url)
        #expect(withoutImage.account.assetImage?.imageURL == nil)
    }

    private func model(_ destination: GemConfirmDestination, addressName: AddressName? = nil) -> ConfirmRowViewModel {
        ConfirmRowViewModel(content: .recipient(destination: destination, addressName: addressName?.toGem(), memo: nil, chain: Chain.ethereum.rawValue, link: BlockExplorerLink.mock().toGem()))
    }
}

private extension ConfirmRowViewModel {
    var recipientItem: AddressListItemViewModel? {
        guard case let .recipient(item) = itemModel else { return nil }
        return item
    }

    var isEmpty: Bool {
        if case .empty = itemModel {
            true
        } else {
            false
        }
    }
}
