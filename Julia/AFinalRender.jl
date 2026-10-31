using Distributions
using FileIO
using Images
using LinearAlgebra
using Random
using Statistics

function convert_to_unit_vector(vector::C
        )::C where {T <: Number, C <: AbstractVector{T}}
    return vector ./ sqrt(dot(vector, vector))
end

function random_in_unit_sphere(T::Type,
        random_generator::AbstractRNG
        )::AbstractArray{T}
    while true
        p = [rand(random_generator, Uniform(-one(T), one(T))),
            rand(random_generator, Uniform(-one(T), one(T))),
            rand(random_generator, Uniform(-one(T), one(T)))]
        if dot(p, p) >= one(T)
            continue
        else
            return p
        end
    end
end

function random_unit_vector(T::Type{<: Number},
    random_generator::AbstractRNG)::AbstractArray{T}
    return convert_to_unit_vector(
        random_in_unit_sphere(T, random_generator))
end

struct Ray{T <: Number, C <: AbstractVector{T}}
    origin::C
    direction::C
end

function fly(ray::Ray{T, C}, t::T
        )::C where {T <: Number, C <: AbstractVector{T}}
    return ray.origin + t * ray.direction
end

abstract type Material end

struct InnerHitRecordWithMat{T <: Number, C <: AbstractVector{T}, D <: Material}
    position::C
    normal::C
    t::T
    material::D
end

struct OuterHitRecordWithMat{T <: Number, C <: AbstractVector{T}, D <: Material}
    position::C
    normal::C
    t::T
    material::D
end

FrontFacedNormalHitRecordWithMat{T, C, D} =
    Union{InnerHitRecordWithMat{T, C, D},
        OuterHitRecordWithMat{T, C, D}
        } where {T <: Number, C <: AbstractVector{T}, D <: Material}

abstract type Hittable end

struct HittablesWithMat
    render_objects::AbstractArray{Hittable}
end

struct SphereWithMat{T <: Number, C <: AbstractVector{T}, D <: Material} <: Hittable
    center::C
    radius::T
    material::D
end

struct Lambertian{T <: AbstractFloat} <: Material
    albedo::RGB{T}
end

function scatter(ray_in::Ray{T, C},
        rec::FrontFacedNormalHitRecordWithMat{T, C, Lambertian{T}},
        random_generator::AbstractRNG
        )::Tuple{Ray{T, C}, RGB{T}, Bool} where {T <: AbstractFloat, C <: AbstractVector{T}}
    scatter_direction = rec.normal + random_unit_vector(T, random_generator)
    return (Ray(rec.position, scatter_direction), rec.material.albedo, true)
end

function reflect(v::C, n::C)::C where {T <: Number, C <: AbstractVector{T}}
    return v - 2dot(v, n)n
end

struct FuzzyMetal{T <: AbstractFloat} <: Material
    albedo::RGB{T}
    fuzziness::T
end

function scatter(ray_in::Ray{T, C},
        rec::FrontFacedNormalHitRecordWithMat{T, C, FuzzyMetal{T}},
        random_generator::AbstractRNG
        )::Tuple{Ray{T, C}, RGB{T}, Bool} where {T <: AbstractFloat, C <: AbstractVector{T}}
    reflected = reflect(convert_to_unit_vector(ray_in.direction), rec.normal)
    scattered = Ray(rec.position, reflected + rec.material.fuzziness * random_in_unit_sphere(T, random_generator))
    return (scattered, rec.material.albedo, dot(scattered.direction, rec.normal) > 0)
end

function refract(uv::C, n::C, etai_over_etat::T)::C where {T <: AbstractFloat, C <: AbstractVector{T}}
    cos_theta = dot(-uv, n)
    r_out_parallel = etai_over_etat * (uv + cos_theta*n)
    r_out_perp = -sqrt(abs(one(T) - dot(r_out_parallel, r_out_parallel))) * n
    return r_out_parallel + r_out_perp
end

struct Dielectric{T} <: Material
    ref_idx::T
end

function schlick(cosine::T, ref_idx::T)::T where T <: Number
    r0 = (one(T) - ref_idx) / (one(T) + ref_idx)
    sqr0 = r0 ^2
    return sqr0 + (one(T) - sqr0)*((one(T) - cosine)^5)
end

function scatter(ray_in::Ray{T, C},
        rec::InnerHitRecordWithMat{T, C, Dielectric{T}},
        random_generator::AbstractRNG
        )::Tuple{Ray{T, C}, RGB{T}, Bool} where {T <: AbstractFloat, C <: AbstractVector{T}}
    pure_color = RGB(one(T), one(T), one(T))
    etai_over_etat = rec.material.ref_idx
    unit_direction = convert_to_unit_vector(ray_in.direction)
    cos_theta = min(dot(-unit_direction, rec.normal), 1.0)
    sin_theta = sqrt(one(T) - cos_theta*cos_theta)
    cannot_refract = etai_over_etat * sin_theta > one(T)
    reflactance = schlick(cos_theta, etai_over_etat)
    if cannot_refract || reflactance > rand(random_generator, T)
        reflected = reflect(unit_direction, rec.normal)
        scattered = Ray(rec.position, reflected)
        return (scattered, pure_color, true)
    else
        refracted = refract(unit_direction, rec.normal, etai_over_etat)
        scattered = Ray(rec.position, refracted)
        return (scattered, pure_color, true)
    end
end

function scatter(ray_in::Ray{T, C},
        rec::OuterHitRecordWithMat{T, C, Dielectric{T}},
        random_generator::AbstractRNG
        )::Tuple{Ray{T, C}, RGB{T}, Bool} where {T <: AbstractFloat, C <: AbstractVector{T}}
    pure_color = RGB(one(T), one(T), one(T))
    etai_over_etat = one(T) / rec.material.ref_idx
    unit_direction = convert_to_unit_vector(ray_in.direction)
    cos_theta = min(dot(-unit_direction, rec.normal), 1.0)
    sin_theta = sqrt(one(T) - cos_theta*cos_theta)
    cannot_refract = etai_over_etat * sin_theta > one(T)
    reflactance = schlick(cos_theta, etai_over_etat)
    if cannot_refract || reflactance > rand(random_generator, T)
        reflected = reflect(unit_direction, rec.normal)
        scattered = Ray(rec.position, reflected)
        return (scattered, pure_color, true)
    else
        refracted = refract(unit_direction, rec.normal, etai_over_etat)
        scattered = Ray(rec.position, refracted)
        return (scattered, pure_color, true)
    end
end

function random_in_unit_disk(T::Type{<: Number}, random_generator::AbstractRNG)
    while true
        p = [rand(random_generator, Uniform(-one(T), one(T))),
                rand(random_generator, Uniform(-one(T), one(T))),
                zero(T)]
        if dot(p, p) >= one(T)
            continue
        else
            return p
        end
    end
end

struct LensCamera{T <: Number, U <: Unsigned, C <: AbstractVector{T}}
    image_height::U
    image_width::U
    origin::C
    horizontal::C
    vertical::C
    lower_left_corner::C
    u::C
    v::C
    w::C
    lens_radius::T
end

function LensCamera(lookfrom::C, lookat::C, vup::C,
        vfov::T,
        image_height::U,
        image_width::U,
        aperture::T,
        focus_dist::T
        )::LensCamera{T, U, C} where {T <: Number, U <: Unsigned, C <: AbstractVector{T}}
    aspect_ratio = T(image_width)/T(image_height)
    theta = deg2rad(vfov)
    h = tan(theta/2)
    viewport_height = 2h
    viewport_width = aspect_ratio * viewport_height

    w = convert_to_unit_vector(lookfrom - lookat)
    u = convert_to_unit_vector(cross(vup,  w))
    v = cross(w, u)

    origin = lookfrom
    horizontal = focus_dist * viewport_width * u
    vertical = focus_dist * viewport_height * v
    lower_left_corner = origin - horizontal ./ 2 - vertical ./ 2 - focus_dist * w

    lens_radius = aperture / 2
    return LensCamera(image_height,
        image_width,
        origin,
        horizontal,
        vertical,
        lower_left_corner,
        u,v,w,
        lens_radius)
end

function calc_uv!(T::Type,
        camera,
        i::U, j::U,
        random_generator::AbstractRNG
        )::Tuple{T, T} where {U <: Unsigned}
    return ((T(i) + rand(random_generator, T) - one(T)) / (camera.image_width - one(T)),
        (camera.image_height + rand(random_generator, T) - T(j)) / (camera.image_height - one(T)))
end

function get_ray(camera::LensCamera,
        s::T, t::T,
        random_generator::AbstractRNG
        )::Ray where {T <: Number}
    rd = camera.lens_radius * random_in_unit_disk(T, random_generator)
    offset = camera.u * rd[1] + camera.v * rd[2]

    return Ray(camera.origin + offset,
        camera.lower_left_corner
        + s .* camera.horizontal
        + t .* camera.vertical
        - camera.origin - offset)
end

function is_face_normal(ray::Ray{T, C},
        outward_normal::C
        )::Bool where {T <: Number, C <: AbstractVector{T}}
    return dot(ray.direction, outward_normal) < zero(T)
end

function hit_within_or(sphere::SphereWithMat{T, C, D},
        ray::Ray{T, C},
        t_min::T, t_max::T, t::T
        )::Union{FrontFacedNormalHitRecordWithMat{T, C, D}, Nothing} where {T <: Number, C <: AbstractVector{T}, D <: Material}
    if t_min < t < t_max
        position = fly(ray, t)
        outward_normal = (position - sphere.center) / sphere.radius
        if is_face_normal(ray, outward_normal)
            return OuterHitRecordWithMat(position, outward_normal, t, sphere.material)
        else
            return InnerHitRecordWithMat(position, -outward_normal, t, sphere.material)
        end
    end
    return
end

function hit_or(sphere::SphereWithMat{T, C, D},
        ray::Ray{T, C},
        t_min::T, t_max::T
        )::Union{FrontFacedNormalHitRecordWithMat{T, C, D}, Nothing} where {T <: Number, C <: AbstractVector{T}, D <: Material}
    oc = ray.origin - sphere.center
    a = dot(ray.direction, ray.direction)
    half_b = dot(oc, ray.direction)
    c = dot(oc, oc) - sphere.radius * sphere.radius
    discriminant = half_b * half_b - a * c
    if discriminant > 0
        root = sqrt(discriminant)
        smaller = (-half_b - root) / a
        return hit_within_or(sphere, ray, t_min, t_max, smaller)
        bigger = (-half_b + root) / a
        return hit_within_or(sphere, ray, t_min, t_max, bigger)
    end
end

function hit_or(hittables::HittablesWithMat,
        ray::Ray{T, C},
        t_min::T, t_max::T
        )::Union{FrontFacedNormalHitRecordWithMat{T, C}, Nothing} where {T <: Number, C <: AbstractVector{T}}
    rec = nothing
    temp_rec = nothing
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

function ray_color(direction::C)::RGB{T} where {T <: AbstractFloat, C <: AbstractVector{T}}
    unit_direction = convert_to_unit_vector(direction)
    parameter = (unit_direction[2] + one(T)) / 2
    return RGB(((one(T) - parameter) * ones(typeof(unit_direction[2]), 3) + parameter * [0.5, 0.7, 1.0])...)
end

function ray_color(hittables::HittablesWithMat,
        ray::Ray{T, C},
        depth::U,
        random_generator::AbstractRNG
        )::RGB{T} where {T <: AbstractFloat, U <: Signed, C <: AbstractVector{T}}
    if depth <= 0
        return RGB(zero(T), zero(T), zero(T))
    end

    rec = hit_or(hittables, ray, 0.001, Inf)
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

function render_color!(hittable::HittablesWithMat,
        camera::LensCamera{T, U},
        max_depth::V,
        random_generator::AbstractRNG,
        buffer::C
        )::C where {T <: AbstractFloat, U <: Unsigned, V <: Signed,
            C <: AbstractMatrix{RGB{T}}}
    for i = 1:camera.image_width
        for j = 1:camera.image_height
            color = ray_color(hittable,
                get_ray(camera,
                    calc_uv!(T, camera, i, j, random_generator)...,
                    random_generator),
                max_depth,
                random_generator)
            buffer[j, i] = color
        end
    end
    return buffer
end

function render_color!(hittable::HittablesWithMat,
        camera::LensCamera{T, U},
        samples_per_pixel::U,
        max_depth::V,
        random_generators::AbstractArray{<: AbstractRNG},
        buffers::C,
        buffer::D
        )::D where {T <: AbstractFloat, U <: Unsigned, V <: Signed,
            C <: AbstractArray{RGB{T}}, D <: AbstractMatrix{RGB{T}}}
        Threads.@threads for s = 1:samples_per_pixel
            buffers[s, :, :] = render_color!(hittable,
                    camera,
                    max_depth,
                    random_generators[s],
                    buffers[s, :, :])
        end

        buffer = reshape(
            mean(buffers, dims=1),
            (camera.image_height, camera.image_width))
        return buffer
end

function random_scene(T::Type{<: AbstractFloat}, random_generator)
    world = []
    ground_material = Lambertian(RGB(0.5, 0.5, 0.5))
    push!(world, SphereWithMat([0.0, -1000.0, 0.0], 1000.0, ground_material))

    for a = -11:11
        for b = -11:11
            material = rand(random_generator, T)
            center = [T(a) + 0.9rand(random_generator, T),
                0.2,
                T(b) + 0.9rand(random_generator, T)]
            diff = center - [4.0, 0.2, 0.0]
            if dot(diff, diff) >= 0.9
                if material < 0.8
                    albedo = RGB(
                        ([rand(random_generator, T),
                            rand(random_generator, T),
                            rand(random_generator, T)] ⊙
                            [rand(random_generator, T),
                                rand(random_generator, T),
                                rand(random_generator, T)])...
                    )
                    push!(world, SphereWithMat(center, 0.2, Lambertian(albedo)))
                elseif material < 0.95
                    albedo = RGB(
                        rand(random_generator, Uniform(0.5, 1.0)),
                            rand(random_generator, Uniform(0.5, 1.0)),
                            rand(random_generator, Uniform(0.5, 1.0))
                    )
                    fuzzyness = rand(random_generator, Uniform(0.0, 0.5))
                    push!(world, SphereWithMat(center, 0.2,
                        FuzzyMetal(albedo, fuzzyness)))
                else
                    push!(world, SphereWithMat(center, 0.2, Dielectric(1.5)))
                end
            end
        end
    end

    push!(world, SphereWithMat([0.0, 1.0, 0.0], 1.0,
        Dielectric(1.5)))
    push!(world, SphereWithMat([-4.0, 1.0, 0.0], 1.0,
        Lambertian(RGB(0.4, 0.2, 0.1))))
    push!(world, SphereWithMat([4.0, 1.0, 0.0], 1.0,
        FuzzyMetal(RGB(0.7, 0.6, 0.5), 0.0)))

    return HittablesWithMat(world)
end

function gamma_correction(color::RGB{T})::RGB{T} where T <: AbstractFloat
    return RGB(sqrt(color.r), sqrt(color.g), sqrt(color.b))
end

function gamma_correction(buffer::AbstractMatrix{RGB{T}}
        )::AbstractMatrix{RGB{T}} where T <: AbstractFloat
    return gamma_correction.(buffer)
end

function main()
    println("PROCESS:Generating the scene...")

    # randoms generators settings
    base_generator = Xoshiro(5103)
    scene_generator = Xoshiro(rand(base_generator, UInt64))
    scene = random_scene(Float64, scene_generator)

    println("NOTICE:The scene has been generated.")
    println("PROCESS:Initializing RNGs...")

    # random generators for each thread
    generators = [Xoshiro(rand(base_generator, UInt64)) for i in 1:samples_per_pixel]

    println("NOTICE:RNGs have been initialized.")

    # rendering buffers
    buffers = zeros(RGB{Float64},
        samples_per_pixel,
        image_height,
        image_width)
    buffer = zeros(RGB{Float64},
        image_height,
        image_width)

    println("PROCESS:Starting rendering the scene...")

    buffer = render_color!(scene,
            lens_camera,
            samples_per_pixel,
            max_depth,
            generators,
            buffers,
            buffer) |> gamma_correction

    println("NOTICE:Rendering has been finished.")
    println("PROCESS:Starting save the buffer.")

    save(ARGS[1], buffer)

    println("FINISHED:All is done!")

end

# lens camera parameters
const lookfrom = [13.0, 2.0, 3.0]
const lookat = [0.0, 0.0, 0.0]
const vup = [0.0, 1.0, 0.0]
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

main()