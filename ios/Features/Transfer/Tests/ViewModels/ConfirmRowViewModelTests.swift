// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAddressRow
import struct Gemstone.GemAvatar
import enum Gemstone.GemConfirmDestination
import enum Gemstone.GemConfirmRowContent
import enum Gemstone.GemListRow
import enum Gemstone.GemLocalizedText
import GemstonePrimitives
import GemstonePrimitivesTestKit
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
        let memo = GemListRow.memo(title: .memo, value: "test memo", menu: [])
        guard case let .row(row) = ConfirmRowViewModel(content: .row(row: memo)).itemModel else {
            Issue.record("Expected a shared row")
            return
        }
        #expect(row == memo)
        #expect(ConfirmRowViewModel(content: .details).isEmpty)
    }

    @Test
    func aRecipientPassesThroughAsCoreBuiltIt() throws {
        let row = GemAddressRow.mock(title: .confirmDestination(destination: .recipient(name: nil, address: "0xrecipient")), address: "0xrecipient", isSelectable: true)
        let item = try #require(ConfirmRowViewModel(content: .recipient(row: row)).recipientItem)

        #expect(item == row)
    }

    @Test
    func aDestinationTitlesItsRow() {
        let cases: [(GemConfirmDestination, String)] = [
            (.recipient(name: nil, address: "0xrecipient"), Localized.Transfer.Recipient.title),
            (.contract(name: nil, address: "0xspender"), Localized.Asset.contract),
            (.validator(name: "Allnodes", address: "validator1"), Localized.Stake.validator),
            (.provider(name: "Yo", address: "0xprovider"), Localized.Common.provider),
            (.resource(resource: Resource.energy.toGem()), Localized.Stake.resource),
        ]
        for (destination, title) in cases {
            #expect(GemLocalizedText.confirmDestination(destination: destination).text == title)
        }
    }

    @Test
    func aContactShowsItsPictureOrTheInitialsCoreWrote() {
        #expect(GemAvatar(imageUrl: "avatar.png", initials: "AD").assetImage.imageURL == ImageSource("avatar.png").url)
        #expect(GemAvatar(imageUrl: nil, initials: "AD").assetImage.imageURL == nil)
        #expect(GemAvatar(imageUrl: nil, initials: "AD").assetImage.type == .text("AD"))
    }
}

private extension ConfirmRowViewModel {
    var recipientItem: GemAddressRow? {
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
