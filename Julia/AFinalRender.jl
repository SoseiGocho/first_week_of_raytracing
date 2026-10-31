using FileIO
using Images
using LinearAlgebra
using Random
using StaticArrays

# SVector の dot / cross / + / * / / は StaticArrays.jl 側で @inline 定義済み。
# ここで再定義すると型パイレートになるため、そのまま利用する。
const Vec3{T} = SVector{3, T}

@inline function convert_to_unit_vector(vector::C
        )::C where {T <: AbstractFloat, C <: AbstractVector{T}}
    return vector / sqrt(dot(vector, vector))
end

@inline function random_in_unit_sphere(::Type{T},
        random_generator::AbstractRNG
        )::Vec3{T} where {T <: AbstractFloat}
    while true
        x = 2rand(random_generator, T) - one(T)
        y = 2rand(random_generator, T) - one(T)
        z = 2rand(random_generator, T) - one(T)
        if x * x + y * y + z * z < one(T)
            return Vec3{T}(x, y, z)
        end
    end
end

@inline function random_unit_vector(::Type{T},
        random_generator::AbstractRNG
        )::Vec3{T} where {T <: AbstractFloat}
    return convert_to_unit_vector(
        random_in_unit_sphere(T, random_generator))
end

struct Ray{T <: AbstractFloat}
    origin::Vec3{T}
    direction::Vec3{T}
end

@inline function fly(ray::Ray{T}, t::T
        )::Vec3{T} where {T <: AbstractFloat}
    return ray.origin + t * ray.direction
end

abstract type Material end

struct Lambertian{T <: AbstractFloat} <: Material
    albedo::RGB{T}
end

struct FuzzyMetal{T <: AbstractFloat} <: Material
    albedo::RGB{T}
    fuzziness::T
end

struct Dielectric{T <: AbstractFloat} <: Material
    ref_idx::T
end

struct HitRecord{T <: AbstractFloat}
    position::Vec3{T}
    normal::Vec3{T}
    t::T
    front_face::Bool
    material::Union{Lambertian{T}, FuzzyMetal{T}, Dielectric{T}}
end

struct SphereWithMat{T <: AbstractFloat, D <: Material}
    center::Vec3{T}
    radius::T
    material::D
end

const Hittable{T} = Union{
    SphereWithMat{T, Lambertian{T}},
    SphereWithMat{T, FuzzyMetal{T}},
    SphereWithMat{T, Dielectric{T}}
}

struct HittablesWithMat{T <: AbstractFloat}
    render_objects::Vector{Hittable{T}}
end

@inline function reflect(v::Vec3{T}, n::Vec3{T}
        )::Vec3{T} where {T <: AbstractFloat}
    return v - 2 * dot(v, n) * n
end

@inline function refract(uv::Vec3{T}, n::Vec3{T}, etai_over_etat::T
        )::Vec3{T} where {T <: AbstractFloat}
    cos_theta = dot(-uv, n)
    r_out_parallel = etai_over_etat * (uv + cos_theta * n)
    r_out_perp = -sqrt(abs(one(T) - dot(r_out_parallel, r_out_parallel))) * n
    return r_out_parallel + r_out_perp
end

@inline function schlick(cosine::T, ref_idx::T
        )::T where {T <: AbstractFloat}
    r0 = (one(T) - ref_idx) / (one(T) + ref_idx)
    sqr0 = r0 ^ 2
    return sqr0 + (one(T) - sqr0) * ((one(T) - cosine) ^ 5)
end

@inline function scatter(ray_in::Ray{T},
        rec::HitRecord{T},
        random_generator::AbstractRNG
        )::Tuple{Ray{T}, RGB{T}, Bool} where {T <: AbstractFloat}
    material = rec.material

    if material isa Lambertian{T}
        scatter_direction = rec.normal + random_unit_vector(T, random_generator)
        if dot(scatter_direction, scatter_direction) < T(1e-8)
            scatter_direction = rec.normal
        end
        return (Ray(rec.position, scatter_direction), material.albedo, true)

    elseif material isa FuzzyMetal{T}
        reflected = reflect(convert_to_unit_vector(ray_in.direction), rec.normal)
        scattered = Ray(rec.position,
            reflected + material.fuzziness * random_in_unit_sphere(T, random_generator))
        return (scattered, material.albedo,
            dot(scattered.direction, rec.normal) > zero(T))

    else
        pure_color = RGB(one(T), one(T), one(T))
        etai_over_etat = rec.front_face ? one(T) / material.ref_idx : material.ref_idx
        unit_direction = convert_to_unit_vector(ray_in.direction)
        cos_theta = min(dot(-unit_direction, rec.normal), one(T))
        sin_theta = sqrt(one(T) - cos_theta * cos_theta)
        cannot_refract = etai_over_etat * sin_theta > one(T)
        reflectance = schlick(cos_theta, etai_over_etat)
        if cannot_refract || reflectance > rand(random_generator, T)
            scattered = Ray(rec.position, reflect(unit_direction, rec.normal))
        else
            scattered = Ray(rec.position,
                refract(unit_direction, rec.normal, etai_over_etat))
        end
        return (scattered, pure_color, true)
    end
end

@inline function random_in_unit_disk(::Type{T},
        random_generator::AbstractRNG
        )::SVector{2, T} where {T <: AbstractFloat}
    while true
        x = 2rand(random_generator, T) - one(T)
        y = 2rand(random_generator, T) - one(T)
        if x * x + y * y < one(T)
            return SVector{2, T}(x, y)
        end
    end
end

struct LensCamera{T <: AbstractFloat, U <: Unsigned}
    image_height::U
    image_width::U
    origin::Vec3{T}
    horizontal::Vec3{T}
    vertical::Vec3{T}
    lower_left_corner::Vec3{T}
    u::Vec3{T}
    v::Vec3{T}
    w::Vec3{T}
    lens_radius::T
end

function LensCamera(lookfrom::Vec3{T}, lookat::Vec3{T}, vup::Vec3{T},
        vfov::T,
        image_height::U,
        image_width::U,
        aperture::T,
        focus_dist::T
        )::LensCamera{T, U} where {T <: AbstractFloat, U <: Unsigned}
    aspect_ratio = T(image_width) / T(image_height)
    theta = deg2rad(vfov)
    h = tan(theta / T(2))
    viewport_height = T(2) * h
    viewport_width = aspect_ratio * viewport_height

    w = convert_to_unit_vector(lookfrom - lookat)
    u = convert_to_unit_vector(cross(vup, w))
    v = cross(w, u)

    origin = lookfrom
    horizontal = focus_dist * viewport_width * u
    vertical = focus_dist * viewport_height * v
    lower_left_corner = origin - horizontal / T(2) - vertical / T(2) - focus_dist * w

    lens_radius = aperture / T(2)
    return LensCamera{T, U}(image_height,
        image_width,
        origin,
        horizontal,
        vertical,
        lower_left_corner,
        u, v, w,
        lens_radius)
end

@inline function calc_uv!(::Type{T},
        camera::LensCamera{T, U},
        i::U, j::U,
        random_generator::AbstractRNG
        )::Tuple{T, T} where {T <: AbstractFloat, U <: Unsigned}
    return ((T(i) + rand(random_generator, T) - one(T)) /
            (T(camera.image_width) - one(T)),
        (T(camera.image_height) + rand(random_generator, T) - T(j)) /
            (T(camera.image_height) - one(T)))
end

@inline function get_ray(camera::LensCamera{T, U},
        s::T, t::T,
        random_generator::AbstractRNG
        )::Ray{T} where {T <: AbstractFloat, U <: Unsigned}
    rd = camera.lens_radius * random_in_unit_disk(T, random_generator)
    offset = camera.u * rd[1] + camera.v * rd[2]

    return Ray(camera.origin + offset,
        camera.lower_left_corner
        + s * camera.horizontal
        + t * camera.vertical
        - camera.origin - offset)
end

@inline function is_face_normal(ray::Ray{T},
        outward_normal::Vec3{T}
        )::Bool where {T <: AbstractFloat}
    return dot(ray.direction, outward_normal) < zero(T)
end

@inline function hit_within_or(sphere::SphereWithMat{T, D},
        ray::Ray{T},
        t_min::T, t_max::T, t::T
        )::Union{HitRecord{T}, Nothing} where {T <: AbstractFloat, D <: Material}
    if t_min < t < t_max
        position = fly(ray, t)
        outward_normal = (position - sphere.center) / sphere.radius
        front_face = is_face_normal(ray, outward_normal)
        normal = front_face ? outward_normal : -outward_normal
        return HitRecord(position, normal, t, front_face, sphere.material)
    end
    return nothing
end

# NOTE: smaller が範囲外の場合は bigger を試す。球の内側から出るレイで
# 2番目の交点を見失わないようにするため。
@inline function hit_or(sphere::SphereWithMat{T, D},
        ray::Ray{T},
        t_min::T, t_max::T
        )::Union{HitRecord{T}, Nothing} where {T <: AbstractFloat, D <: Material}
    oc = ray.origin - sphere.center
    a = dot(ray.direction, ray.direction)
    half_b = dot(oc, ray.direction)
    c = dot(oc, oc) - sphere.radius * sphere.radius
    discriminant = half_b * half_b - a * c
    if discriminant > zero(T)
        root = sqrt(discriminant)
        smaller = (-half_b - root) / a
        rec = hit_within_or(sphere, ray, t_min, t_max, smaller)
        if !isnothing(rec)
            return rec
        end
        bigger = (-half_b + root) / a
        return hit_within_or(sphere, ray, t_min, t_max, bigger)
    end
    return nothing
end

function hit_or(hittables::HittablesWithMat{T},
        ray::Ray{T},
        t_min::T, t_max::T
        )::Union{HitRecord{T}, Nothing} where {T <: AbstractFloat}
    rec = nothing
    closest_so_far = t_max
    for target ∈ hittables.render_objects
        temp_rec = hit_or(target, ray, t_min, closest_so_far)
        if !isnothing(temp_rec)
            rec = temp_rec
            closest_so_far = temp_rec.t
        end
    end
    return rec
end

@inline function ray_color(direction::Vec3{T}
        )::RGB{T} where {T <: AbstractFloat}
    unit_direction = convert_to_unit_vector(direction)
    parameter = (unit_direction[2] + one(T)) / T(2)
    return RGB(
        (one(T) - parameter) + parameter * T(0.5),
        (one(T) - parameter) + parameter * T(0.7),
        (one(T) - parameter) + parameter * one(T))
end

function ray_color(hittables::HittablesWithMat{T},
        ray::Ray{T},
        depth::U,
        random_generator::AbstractRNG
        )::RGB{T} where {T <: AbstractFloat, U <: Signed}
    if depth <= 0
        return RGB(zero(T), zero(T), zero(T))
    end

    rec = hit_or(hittables, ray, T(0.001), T(Inf))
    if isnothing(rec)
        return ray_color(ray.direction)
    else
        scattered_ray, attenuation, is_scattered = scatter(ray, rec, random_generator)
        if is_scattered
            color = ray_color(hittables, scattered_ray, depth - one(U), random_generator)
            return attenuation ⊙ color
        else
            return RGB(zero(T), zero(T), zero(T))
        end
    end
end

function render_color!(hittable::HittablesWithMat{T},
        camera::LensCamera{T, U},
        samples_per_pixel::U,
        max_depth::V,
        seed::UInt64,
        buffer::C
        )::C where {T <: AbstractFloat, U <: Unsigned, V <: Signed,
            C <: AbstractMatrix{RGB{T}}}
    image_height, image_width = size(buffer)
    progress = Threads.Atomic{Int}(0)
    progress_step = max(1, image_height ÷ 20)
    progress_lock = ReentrantLock()

    Threads.@threads :static for j in 1:image_height
        random_generator = Xoshiro(seed + UInt64(j))
        for i in 1:image_width
            color = zero(RGB{T})
            for _ in 1:samples_per_pixel
                color += ray_color(hittable,
                    get_ray(camera,
                        calc_uv!(T, camera, UInt(i), UInt(j), random_generator)...,
                        random_generator),
                    max_depth,
                    random_generator)
            end
            @inbounds buffer[j, i] = color / T(samples_per_pixel)
        end

        lock(progress_lock) do
            done = Threads.atomic_add!(progress, 1) + 1
            if done % progress_step == 0 || done == image_height
                println("PROGRESS: $(done)/$(image_height) rows " *
                    "($(round(100 * done / image_height, digits=1))%)")
            end
        end
    end

    return buffer
end

function random_scene(::Type{T}, random_generator::AbstractRNG
        )::HittablesWithMat{T} where {T <: AbstractFloat}
    world = Hittable{T}[]
    ground_material = Lambertian(RGB(T(0.5), T(0.5), T(0.5)))
    push!(world, SphereWithMat(
        Vec3{T}(T(0), T(-1000), T(0)), T(1000), ground_material))

    for a = -11:11
        for b = -11:11
            material = rand(random_generator, T)
            center = Vec3{T}(
                T(a) + T(0.9) * rand(random_generator, T),
                T(0.2),
                T(b) + T(0.9) * rand(random_generator, T))
            diff = center - Vec3{T}(T(4), T(0.2), T(0))
            if dot(diff, diff) >= T(0.9)
                if material < T(0.8)
                    albedo = RGB(
                        rand(random_generator, T) * rand(random_generator, T),
                        rand(random_generator, T) * rand(random_generator, T),
                        rand(random_generator, T) * rand(random_generator, T))
                    push!(world, SphereWithMat(center, T(0.2), Lambertian(albedo)))
                elseif material < T(0.95)
                    albedo = RGB(
                        T(0.5) + T(0.5) * rand(random_generator, T),
                        T(0.5) + T(0.5) * rand(random_generator, T),
                        T(0.5) + T(0.5) * rand(random_generator, T))
                    fuzzyness = T(0.5) * rand(random_generator, T)
                    push!(world, SphereWithMat(center, T(0.2),
                        FuzzyMetal(albedo, fuzzyness)))
                else
                    push!(world, SphereWithMat(center, T(0.2),
                        Dielectric(T(1.5))))
                end
            end
        end
    end

    push!(world, SphereWithMat(Vec3{T}(T(0), T(1), T(0)), T(1),
        Dielectric(T(1.5))))
    push!(world, SphereWithMat(Vec3{T}(T(-4), T(1), T(0)), T(1),
        Lambertian(RGB(T(0.4), T(0.2), T(0.1)))))
    push!(world, SphereWithMat(Vec3{T}(T(4), T(1), T(0)), T(1),
        FuzzyMetal(RGB(T(0.7), T(0.6), T(0.5)), T(0.0))))

    return HittablesWithMat{T}(world)
end

@inline function gamma_correction(color::RGB{T}
        )::RGB{T} where {T <: AbstractFloat}
    return RGB(sqrt(color.r), sqrt(color.g), sqrt(color.b))
end

function gamma_correction(buffer::AbstractMatrix{RGB{T}}
        ) where {T <: AbstractFloat}
    return gamma_correction.(buffer)
end

function usage()
    println("Usage: julia --project=. -t <threads> AFinalRender.jl <output-image>")
    println("Example: julia --project=. -t 16 AFinalRender.jl outputs/final.png")
end

function main()
    if length(ARGS) != 1
        usage()
        return
    end
    output_path = ARGS[1]

    T = Float64
    println("PROCESS:Generating the scene...")

    # random generators settings
    base_generator = Xoshiro(5103)
    scene_generator = Xoshiro(rand(base_generator, UInt64))
    scene = random_scene(T, scene_generator)

    println("NOTICE:The scene has been generated.")
    println("PROCESS:Initializing the RNG seed...")

    render_seed = rand(base_generator, UInt64)

    println("NOTICE:The RNG seed has been initialized.")

    # rendering buffer
    buffer = zeros(RGB{T}, image_height, image_width)

    println("PROCESS:Starting rendering the scene...")

    render_color!(scene,
        lens_camera,
        samples_per_pixel,
        max_depth,
        render_seed,
        buffer)

    println("NOTICE:Rendering has been finished.")
    println("PROCESS:Starting save the buffer.")

    save(output_path, gamma_correction(buffer))

    println("FINISHED:All is done!")
end

# lens camera parameters
const lookfrom = Vec3{Float64}(13.0, 2.0, 3.0)
const lookat = Vec3{Float64}(0.0, 0.0, 0.0)
const vup = Vec3{Float64}(0.0, 1.0, 0.0)
const vfov = 20.0
const image_height = UInt(720)
const image_width = UInt(1280)
const aperture = 0.1
const dist_to_focus = 10.0
const lens_camera = LensCamera(lookfrom,
    lookat,
    vup,
    vfov,
    image_height,
    image_width,
    aperture,
    dist_to_focus)
# rendering parameters
const samples_per_pixel = UInt(128)
const max_depth = 64

if abspath(PROGRAM_FILE) == abspath(@__FILE__)
    main()
end
