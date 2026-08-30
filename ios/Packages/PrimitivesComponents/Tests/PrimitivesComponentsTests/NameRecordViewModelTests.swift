// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

@MainActor
struct NameRecordViewModelTests {
    @Test
    func acceptsOnlyCompleteRecords() async throws {
        let record = NameRecord.mock()
        let nameService = GemNameServiceMock(nameRecord: record)
        let valid = NameRecordViewModel(nameService: nameService)
        let emptyAddress = NameRecordViewModel(nameService: GemNameServiceMock(nameRecord: .mock(address: "")))
        let emptyName = NameRecordViewModel(nameService: GemNameServiceMock(nameRecord: .mock(name: "")))

        valid.getNameRecord(name: record.name, chain: record.chain)
        emptyAddress.getNameRecord(name: record.name, chain: record.chain)
        emptyName.getNameRecord(name: record.name, chain: record.chain)
        try await Task.sleep(for: .milliseconds(500))

        #expect(valid.state == .complete(record))
        #expect(emptyAddress.state == .error)
        #expect(emptyName.state == .error)

        valid.getNameRecord(name: record.name, chain: record.chain)

        #expect(valid.state == .complete(record))
        #expect(nameService.requestedNames == [record.name])
    }
}
