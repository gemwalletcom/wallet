// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Testing

struct NetworkSelectorViewModelTests {
    private let model = NetworkSelectorViewModel(
        state: .data(.plain([.bitcoin, .ethereum])),
        selectedItems: [],
        selectionType: .checkmark,
    )

    @Test
    func searchable() {
        #expect(model.search != nil)
        #expect(model.items == [.bitcoin, .ethereum])
    }

    @Test
    func filtersByQuery() throws {
        let search = try #require(model.search)

        #expect(search.filter(.bitcoin, "bitc"))
        #expect(!search.filter(.bitcoin, "ethereum"))
    }
}
