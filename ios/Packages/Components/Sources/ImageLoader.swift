// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import ImageIO
import UIKit

struct ImageRequest: Hashable, Sendable {
    let url: URL
    let maxPixelSize: CGFloat?
    let scale: CGFloat

    var cacheKey: NSString {
        "\(url.absoluteString)#\(Int(maxPixelSize?.rounded(.up) ?? 0))@\(scale)" as NSString
    }
}

enum ImageLoadingError: Error {
    case undecodable
}

final class ImageLoader: @unchecked Sendable {
    static let shared = ImageLoader()

    private static let decodedImagesLimit = 64 * 1024 * 1024

    private let session: URLSession
    private let images = NSCache<NSString, UIImage>()
    private let loads = InFlightLoads()

    init(cache: URLCache = .shared) {
        let configuration = URLSessionConfiguration.default
        configuration.urlCache = cache
        session = URLSession(configuration: configuration)
        images.totalCostLimit = Self.decodedImagesLimit
    }

    func cached(_ request: ImageRequest) -> UIImage? {
        images.object(forKey: request.cacheKey)
    }

    func image(for request: ImageRequest) async throws -> UIImage {
        if let image = cached(request) {
            return image
        }
        return try await loads.task(for: request) { try await self.load(request) }.value
    }

    private func load(_ request: ImageRequest) async throws -> UIImage {
        let data = try await data(for: URLRequest(url: request.url))
        guard let image = Self.decode(data, request: request) else {
            throw ImageLoadingError.undecodable
        }
        images.setObject(image, forKey: request.cacheKey, cost: image.cgImage.map { $0.bytesPerRow * $0.height } ?? 0)
        return image
    }

    private func data(for request: URLRequest) async throws -> Data {
        let cache = session.configuration.urlCache
        if let cached = cache?.cachedResponse(for: request) {
            return cached.data
        }
        let (data, response) = try await session.data(for: request)
        if cache?.cachedResponse(for: request) == nil {
            cache?.storeCachedResponse(CachedURLResponse(response: response, data: data), for: request)
        }
        return data
    }

    static func decode(_ data: Data, request: ImageRequest) -> UIImage? {
        guard let source = CGImageSourceCreateWithData(data as CFData, nil) else {
            return nil
        }
        let image: CGImage? = if let maxPixelSize = request.maxPixelSize {
            CGImageSourceCreateThumbnailAtIndex(source, 0, [
                kCGImageSourceCreateThumbnailFromImageAlways: true,
                kCGImageSourceCreateThumbnailWithTransform: true,
                kCGImageSourceShouldCacheImmediately: true,
                kCGImageSourceThumbnailMaxPixelSize: Int(maxPixelSize.rounded(.up)),
            ] as CFDictionary)
        } else {
            CGImageSourceCreateImageAtIndex(source, 0, [kCGImageSourceShouldCacheImmediately: true] as CFDictionary)
        }
        return image.map { UIImage(cgImage: $0, scale: request.scale, orientation: .up) }
    }
}

private actor InFlightLoads {
    private var tasks: [ImageRequest: Task<UIImage, Error>] = [:]

    func task(for request: ImageRequest, load: @escaping @Sendable () async throws -> UIImage) -> Task<UIImage, Error> {
        if let task = tasks[request] {
            return task
        }
        let task = Task { try await load() }
        tasks[request] = task
        Task {
            _ = await task.result
            tasks[request] = nil
        }
        return task
    }
}
