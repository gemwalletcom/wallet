// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmLoad
import enum Gemstone.GemExecuteResult
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
import StoreTestKit
@testable import Transfer
import TransferTestKit
import struct Gemstone.SimulationResult

@MainActor
extension ConfirmTransferSceneViewModel {
    static func mock(
        request: ConfirmTransferRequest? = nil,
        wallet: Wallet? = nil,
        data: GemTransferData = .mock(),
        simulation: SimulationResult? = nil,
        gemConfirmService: GemConfirmServiceMock = GemConfirmServiceMock(),
        load: Result<GemConfirmLoad, any Error> = .success(.mock()),
        execute: Result<GemExecuteResult, any Error> = .success(.signed(data: [])),
        onComplete: VoidAction = nil,
    ) -> ConfirmTransferSceneViewModel {
        let wallet = wallet ?? .mock(accounts: [.mock(chain: data.chain)])
        return ConfirmTransferSceneViewModel(
            request: request ?? ConfirmTransferRequest(data: data, simulation: simulation),
            wallet: wallet,
            session: GemConfirmSessionMock(
                state: .mock(feeAsset: data.feeAsset().map(), simulation: gemConfirmService.simulation, preload: nil),
                load: load,
                execute: execute,
            ),
            onComplete: onComplete,
        )
    }
}
