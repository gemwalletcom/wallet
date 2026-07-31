// Copyright (c). Gem Wallet. All rights reserved.

import AddressNameService
import AssetsService
import BigInt
import Primitives
import PrimitivesComponents

public struct ConfirmSimulationService: Sendable {
    private let addressNameService: AddressNameService
    private let assetsService: AssetsService

    public init(
        addressNameService: AddressNameService,
        assetsService: AssetsService,
    ) {
        self.addressNameService = addressNameService
        self.assetsService = assetsService
    }

    func makeState(data: TransferData, simulation: SimulationResult?) -> ConfirmSimulationState {
        let assets = simulationAssets(simulation)
        return buildState(
            simulation: simulation,
            payload: payloadFields(for: data.type, simulation: simulation),
            payloadAddressNames: [:],
            headerData: cachedHeaderData(data: data, simulation: simulation, assets: assets),
            balanceChanges: balanceChanges(simulation: simulation, assets: assets),
        )
    }

    func updateState(data: TransferData, simulation: SimulationResult?) async -> ConfirmSimulationState {
        let payload = payloadFields(for: data.type, simulation: simulation)
        async let names = payloadAddressNames(chain: data.chain, payload: payload)
        do {
            try await assetsService.prefetchAssets(assetIds: simulation?.simulationAssetIds ?? [])
        } catch {
            debugLog("simulation asset prefetch error: \(error)")
        }

        let assets = simulationAssets(simulation)
        return await buildState(
            simulation: simulation,
            payload: payload,
            payloadAddressNames: names,
            headerData: cachedHeaderData(data: data, simulation: simulation, assets: assets),
            balanceChanges: balanceChanges(simulation: simulation, assets: assets),
        )
    }
}

private extension ConfirmSimulationService {
    func buildState(
        simulation: SimulationResult?,
        payload: [SimulationPayloadField],
        payloadAddressNames: [ChainAddress: AddressName],
        headerData: AssetValueHeaderData?,
        balanceChanges: [SimulationAssetChange],
    ) -> ConfirmSimulationState {
        ConfirmSimulationState(
            warnings: simulation?.warnings ?? [],
            primaryFields: payload.filter { $0.display == .primary },
            secondaryFields: payload.filter { $0.display == .secondary },
            payloadAddressNames: payloadAddressNames,
            headerData: headerData,
            balanceChanges: balanceChanges,
        )
    }

    func payloadFields(
        for transferType: TransferDataType,
        simulation: SimulationResult?,
    ) -> [SimulationPayloadField] {
        guard case .generic = transferType else {
            return []
        }

        let payload = simulation?.payload ?? []
        guard shouldHideValueField(for: transferType, simulation: simulation) else {
            return payload
        }

        return payload.filter { $0.kind != .value }
    }

    func approvalHeaderData(for transferType: TransferDataType) -> AssetValueHeaderData? {
        guard case let .tokenApprove(asset, approval) = transferType,
              let value = approval.approvalValue
        else {
            return nil
        }

        return AssetValueHeaderData(asset: asset, value: value)
    }

    func cachedHeaderData(
        data: TransferData,
        simulation: SimulationResult?,
        assets: [AssetId: Asset],
    ) -> AssetValueHeaderData? {
        if let headerData = approvalHeaderData(for: data.type) {
            return headerData
        }

        guard case .generic = data.type,
              let headerValue = simulationHeaderValue(simulation)
        else {
            return nil
        }

        guard let asset = assets[headerValue.assetId] else {
            return nil
        }
        return AssetValueHeaderData(asset: asset, value: headerValue.value)
    }

    func balanceChanges(
        simulation: SimulationResult?,
        assets: [AssetId: Asset],
    ) -> [SimulationAssetChange] {
        (simulation?.balanceChanges ?? []).compactMap { change in
            guard let value = BigInt(change.value, radix: 10),
                  value != .zero,
                  let asset = assets[change.assetId]
            else {
                return nil
            }
            return SimulationAssetChange(
                asset: asset,
                value: value,
            )
        }
    }

    func simulationAssets(_ simulation: SimulationResult?) -> [AssetId: Asset] {
        guard let simulation else {
            return [:]
        }
        let assets = (try? assetsService.getAssets(for: simulation.simulationAssetIds)) ?? []
        return Dictionary(uniqueKeysWithValues: assets.map { ($0.id, $0) })
    }

    func payloadAddressNames(chain: Chain, payload: [SimulationPayloadField]) async -> [ChainAddress: AddressName] {
        let requests = payloadAddressRequests(chain: chain, payload: payload)

        do {
            return try await addressNameService.getAddressNames(requests: requests)
        } catch {
            if !error.isCancelled {
                debugLog("payload address name lookup error: \(error)")
            }
            return [:]
        }
    }

    func payloadAddressRequests(chain: Chain, payload: [SimulationPayloadField]) -> [ChainAddress] {
        payload.compactMap {
            guard $0.fieldType == .address else {
                return nil
            }
            return ChainAddress(chain: chain, address: $0.value)
        }
    }

    func shouldHideValueField(for transferType: TransferDataType, simulation: SimulationResult?) -> Bool {
        if approvalHeaderData(for: transferType) != nil {
            return true
        }

        return simulationHeaderValue(simulation) != nil
    }

    func simulationHeaderValue(_ simulation: SimulationResult?) -> (assetId: AssetId, value: ApprovalValue)? {
        guard let header = simulation?.header,
              let value = header.approvalValue
        else {
            return nil
        }
        return (header.assetId, value)
    }
}

private extension SimulationResult {
    var simulationAssetIds: [AssetId] {
        balanceChanges.map(\.assetId) + [header?.assetId].compactMap(\.self)
    }
}
