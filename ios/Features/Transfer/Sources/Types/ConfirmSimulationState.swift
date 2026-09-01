// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemConfirmSimulation
import GemstonePrimitives
import Primitives
import PrimitivesComponents

struct ConfirmSimulationState {
    let result: SimulationResult?
    let warnings: [SimulationWarning]
    let payload: SimulationPayloadModel
    let headerData: AssetValueHeaderData?
    let balanceChanges: [SimulationAssetChange]

    init(
        result: SimulationResult?,
        warnings: [SimulationWarning],
        payload: SimulationPayloadModel,
        headerData: AssetValueHeaderData?,
        balanceChanges: [SimulationAssetChange],
    ) {
        self.result = result
        self.warnings = warnings
        self.payload = payload
        self.headerData = headerData
        self.balanceChanges = balanceChanges
    }

    init(
        data: TransferData,
        simulation: SimulationResult?,
        resolved: GemConfirmSimulation?,
        addressNames: [ChainAddress: AddressName],
    ) {
        let fields = resolved?.payloadFields.compactMap { try? SimulationPayloadField($0) } ?? []
        var payload = SimulationPayloadModel(
            chain: data.chain,
            primaryFields: fields.primaryFields,
            secondaryFields: fields.secondaryFields,
        )
        payload.addressNames = addressNames
        self.init(
            result: simulation,
            warnings: simulation?.warnings ?? [],
            payload: payload,
            headerData: resolved?.header.flatMap { header in
                guard let value = try? header.value.map() else { return nil }
                return AssetValueHeaderData(asset: header.asset.map(), value: value)
            },
            balanceChanges: resolved?.balanceChanges.compactMap { change in
                guard let value = BigInt(change.value, radix: 10) else { return nil }
                return SimulationAssetChange(asset: change.asset.map(), value: value)
            } ?? [],
        )
    }
}
