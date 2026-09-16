// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.AddressName
import struct Gemstone.GemConfirmLoad
import enum Gemstone.GemConfirmRowContent
import enum Gemstone.GemExecuteResult
import struct Gemstone.GemTransferData
import struct Gemstone.SimulationResult
import func Gemstone.walletRow
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Transfer

public extension ConfirmTransferSceneViewModel {
    static func mock(
        request: ConfirmTransferRequest? = nil,
        data: GemTransferData = .mock(),
        simulation: SimulationResult? = nil,
        load: Result<GemConfirmLoad, any Error>? = nil,
        execute: Result<GemExecuteResult, any Error> = .success(.signed(data: [], warning: nil)),
        rows: ((Gemstone.AddressName?) -> [GemConfirmRowContent])? = nil,
        confirmation: GemConfirmationMock? = nil,
        onComplete: ((GemExecuteResult) -> Void)? = nil,
    ) -> ConfirmTransferSceneViewModel {
        let wallet = Wallet.mock(accounts: [.mock(chain: data.chain)])
        let rows = rows ?? { addressName in
            [
                .sender(wallet: walletRow(wallet: wallet.toGem())),
                .recipient(
                    destination: .recipient(name: addressName?.name, address: data.recipient.address),
                    addressName: addressName,
                    memo: data.recipient.memo,
                    chain: data.chain.rawValue,
                    link: BlockExplorerLink.mock().toGem(),
                ),
                .network(chain: data.chain.rawValue, name: data.chain.rawValue),
                data.recipient.memo.map { GemConfirmRowContent.memo(memo: $0) },
                .details,
            ].compactMap(\.self)
        }
        return ConfirmTransferSceneViewModel(
            request: request ?? .mock(data: data, simulation: simulation),
            wallet: wallet,
            confirmation: confirmation ?? GemConfirmationMock(
                state: .mock(transfer: data, feeAsset: data.feeAsset().toPrimitives(), preload: nil),
                load: load ?? .success(.mock(transfer: data)),
                execute: execute,
                rows: rows,
            ),
            onComplete: onComplete,
        )
    }
}
