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
        if let image = images.object(forKey: request.cacheKey) {
            return image
        }
        guard let cached = session.configuration.urlCache?.cachedResponse(for: URLRequest(url: request.url)),
              let image = Self.decode(cached.data, request: request)
        else {
            return nil
        }
        return store(image, for: request)
    }

    func image(for request: ImageRequest) async throws -> UIImage {
        if let image = cached(request) {
            return image
        }
        return try await loads.task(for: request) { try await self.load(request) }.value
    }

    private func load(_ request: ImageRequest) async throws -> UIImage {
        let urlRequest = URLRequest(url: request.url)
        let cache = session.configuration.urlCache
        let (data, response) = try await session.data(for: urlRequest)
        guard let image = Self.decode(data, request: request) else {
            throw ImageLoadingError.undecodable
        }
        if cache?.cachedResponse(for: urlRequest) == nil {
            cache?.storeCachedResponse(CachedURLResponse(response: response, data: data), for: urlRequest)
        }
        return store(image, for: request)
    }

    private func store(_ image: UIImage, for request: ImageRequest) -> UIImage {
        images.setObject(image, forKey: request.cacheKey, cost: image.cgImage.map { $0.bytesPerRow * $0.height } ?? 0)
        return image
    }

    static func decode(_ data: Data, request: ImageRequest) -> UIImage? {
        guard let source = CGImageSourceCreateWithData(data as CFData, nil),
              let maxPixelSize = request.maxPixelSize.map({ Int($0.rounded(.up)) }) ?? sourcePixelSize(source)
        else {
            return nil
        }
        let image = CGImageSourceCreateThumbnailAtIndex(source, 0, [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true,
            kCGImageSourceShouldCacheImmediately: true,
            kCGImageSourceThumbnailMaxPixelSize: maxPixelSize,
        ] as CFDictionary)
        return image.map { UIImage(cgImage: $0, scale: request.scale, orientation: .up) }
    }

    private static func sourcePixelSize(_ source: CGImageSource) -> Int? {
        guard let properties = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any],
              let width = properties[kCGImagePropertyPixelWidth] as? Int,
              let height = properties[kCGImagePropertyPixelHeight] as? Int
        else {
            return nil
        }
        return max(width, height)
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
