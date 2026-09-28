// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import ImageGalleryService

public struct ImageGallerySaverMock: ImageGallerySaving {
    private let result: @Sendable () async -> ImageGalleryServiceError?

    public init(result: @escaping @Sendable () async -> ImageGalleryServiceError? = { nil }) {
        self.result = result
    }

    public func saveImageFromURL(_: URL) async throws(ImageGalleryServiceError) {
        if let error = await result() {
            throw error
        }
    }
}
