// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.NameRecord
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

@MainActor
struct NameRecordViewModelTests {
    @Test
    func keepsTheResolvedStateAndSkipsARepeatedName() async {
        let chain = Chain.ethereum
        let record = NameRecord.mock(name: "test.eth", chain: chain.rawValue, address: "0x1234567890123456789012345678901234567890")
        let nameService = GemNameServiceMock(nameRecord: record)
        let model = NameRecordViewModel(nameService: nameService)

        model.getNameRecord(name: record.name, chain: chain)
        await model.nameRecordTask?.value

        #expect(model.state == .complete(record: record))

        model.getNameRecord(name: record.name, chain: chain)

        #expect(model.state == .complete(record: record))
        #expect(nameService.requestedNames == [record.name])
    }

    @Test
    func failedLookupEndsInError() async {
        let model = NameRecordViewModel(nameService: GemNameServiceMock(error: AnyError("offline")))

        model.getNameRecord(name: "test.eth", chain: .ethereum)
        await model.nameRecordTask?.value

        #expect(model.state == .error)
    }
}
