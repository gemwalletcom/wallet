// Copyright (c). Gem Wallet. All rights reserved.

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
        let record = NameRecord.mock(name: "test.eth", chain: .ethereum, address: "0x1234567890123456789012345678901234567890")
        let nameService = GemNameServiceMock(nameRecord: record)
        let model = NameRecordViewModel(nameService: nameService)

        model.getNameRecord(name: record.name, chain: record.chain)
        await model.nameRecordTask?.value

        #expect(model.state == .complete(record: record.toGem()))

        model.getNameRecord(name: record.name, chain: record.chain)

        #expect(model.state == .complete(record: record.toGem()))
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
