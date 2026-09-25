// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.addressCopy
import struct Gemstone.AddressName
import class Gemstone.GemAddressService
import struct Gemstone.GemConfirmLoad
import enum Gemstone.GemConfirmRowContent
import enum Gemstone.GemSubmitResult
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
        execute: Result<GemSubmitResult, any Error> = .success(.signed(data: [], message: nil)),
        rows: ((Gemstone.AddressName?) -> [GemConfirmRowContent])? = nil,
        confirmation: GemConfirmationMock? = nil,
        onComplete: ((GemSubmitResult) -> Void)? = nil,
    ) -> ConfirmTransferSceneViewModel {
        let wallet = Wallet.mock(accounts: [.mock(chain: data.chain)])
        let rows = rows ?? { addressName in
            let walletContent: GemConfirmRowContent = .row(row: .wallet(
                wallet: walletRow(wallet: wallet.toGem()),
                copy: addressCopy(chain: data.chain.rawValue, address: wallet.accounts[0].address),
                explorer: BlockExplorerLink.mock().toGem(),
            ))
            let recipient: GemConfirmRowContent = .recipient(
                destination: .recipient(name: addressName?.name, address: data.recipient.address),
                name: addressName?.name,
                text: addressName?.name ?? GemAddressService.shared.format(address: data.recipient.address, chain: data.chain, style: .short),
                address: data.recipient.address,
                memo: data.recipient.memo,
                chain: data.chain.rawValue,
                link: BlockExplorerLink.mock().toGem(),
                avatar: nil,
                isSelectable: true,
            )
            let network: GemConfirmRowContent = .row(row: .network(title: .network, chain: data.chain.rawValue, name: data.chain.rawValue))
            let memo: GemConfirmRowContent? = data.recipient.memo.map { .row(row: .memo(value: $0, copy: $0)) }
            return [walletContent, recipient, network, memo, .details].compactMap(\.self)
        }
        return ConfirmTransferSceneViewModel(
            request: request ?? .mock(data: data, simulation: simulation),
            wallet: wallet,
            confirmation: confirmation ?? GemConfirmationMock(
                state: .mock(transfer: data, feeAsset: data.feeAsset(), fee: nil),
                load: load ?? .success(.mock(transfer: data, fee: .mock())),
                execute: execute,
                rows: rows,
            ),
            onComplete: onComplete,
        )
    }
}
