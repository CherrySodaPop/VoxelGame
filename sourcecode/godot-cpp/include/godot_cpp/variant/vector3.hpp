/*************************************************************************/
/*  vector3.hpp                                                          */
/*************************************************************************/
/*                       This file is part of:                           */
/*                           GODOT ENGINE                                */
/*                      https://godotengine.org                          */
/*************************************************************************/
/* Copyright (c) 2007-2022 Juan Linietsky, Ariel Manzur.                 */
/* Copyright (c) 2014-2022 Godot Engine contributors (cf. AUTHORS.md).   */
/*                                                                       */
/* Permission is hereby granted, free of charge, to any person obtaining */
/* a copy of this software and associated documentation files (the       */
/* "Software"), to deal in the Software without restriction, including   */
/* without limitation the rights to use, copy, modify, merge, publish,   */
/* distribute, sublicense, and/or sell copies of the Software, and to    */
/* permit persons to whom the Software is furnished to do so, subject to */
/* the following conditions:                                             */
/*                                                                       */
/* The above copyright notice and this permission notice shall be        */
/* included in all copies or substantial portions of the Software.       */
/*                                                                       */
/* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,       */
/* EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF    */
/* MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.*/
/* IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY  */
/* CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,  */
/* TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE     */
/* SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.                */
/*************************************************************************/

#ifndef GODOT_VECTOR3_HPP
#define GODOT_VECTOR3_HPP

#include <godot_cpp/core/math.hpp>

namespace godot {

class Basis;
class String;
class Vector3i;

class Vector3 {
	_FORCE_INLINE_ GDNativeTypePtr _native_ptr() const { return (void *)this; }

	friend class Variant;

public:
	enum Axis {
		AXIS_X,
		AXIS_Y,
		AXIS_Z,
	};

	union {
		struct {
			real_t x;
			real_t y;
			real_t z;
		};

		real_t coord[3] = { 0 };
	};

	inline const real_t &operator[](int p_axis) const {
		return coord[p_axis];
	}

	inline real_t &operator[](int p_axis) {
		return coord[p_axis];
	}

	void set_axis(int p_axis, real_t p_value);
	real_t get_axis(int p_axis) const;

	int min_axis() const;
	int max_axis() const;

	inline real_t length() const;
	inline real_t length_squared() const;

	inline void normalize();
	inline Vector3 normalized() const;
	inline bool is_normalized() const;
	inline Vector3 inverse() const;

	inline void zero();

	void snap(Vector3 p_val);
	Vector3 snapped(Vector3 p_val) const;

	void rotate(const Vector3 &p_axis, real_t p_phi);
	Vector3 rotated(const Vector3 &p_axis, real_t p_phi) const;

	/* Static Methods between 2 vector3s */

	inline Vector3 lerp(const Vector3 &p_to, real_t p_weight) const;
	inline Vector3 slerp(const Vector3 &p_to, real_t p_weight) const;
	Vector3 cubic_interpolate(const Vector3 &p_b, const Vector3 &p_pre_a, const Vector3 &p_post_b, real_t p_weight) const;
	Vector3 move_toward(const Vector3 &p_to, const real_t p_delta) const;

	inline Vector3 cross(const Vector3 &p_b) const;
	inline real_t dot(const Vector3 &p_b) const;
	Basis outer(const Vector3 &p_b) const;
	Basis to_diagonal_matrix() const;

	inline Vector3 abs() const;
	inline Vector3 floor() const;
	inline Vector3 sign() const;
	inline Vector3 ceil() const;
	inline Vector3 round() const;

	inline real_t distance_to(const Vector3 &p_to) const;
	inline real_t distance_squared_to(const Vector3 &p_to) const;

	inline Vector3 posmod(const real_t p_mod) const;
	inline Vector3 posmodv(const Vector3 &p_modv) const;
	inline Vector3 project(const Vector3 &p_to) const;

	inline real_t angle_to(const Vector3 &p_to) const;
	inline Vector3 direction_to(const Vector3 &p_to) const;

	inline Vector3 slide(const Vector3 &p_normal) const;
	inline Vector3 bounce(const Vector3 &p_normal) const;
	inline Vector3 reflect(const Vector3 &p_normal) const;

	bool is_equal_approx(const Vector3 &p_v) const;

	/* Operators */

	inline Vector3 &operator+=(const Vector3 &p_v);
	inline Vector3 operator+(const Vector3 &p_v) const;
	inline Vector3 &operator-=(const Vector3 &p_v);
	inline Vector3 operator-(const Vector3 &p_v) const;
	inline Vector3 &operator*=(const Vector3 &p_v);
	inline Vector3 operator*(const Vector3 &p_v) const;
	inline Vector3 &operator/=(const Vector3 &p_v);
	inline Vector3 operator/(const Vector3 &p_v) const;

	inline Vector3 &operator*=(real_t p_scalar);
	inline Vector3 operator*(real_t p_scalar) const;
	inline Vector3 &operator/=(real_t p_scalar);
	inline Vector3 operator/(real_t p_scalar) const;

	inline Vector3 operator-() const;

	inline bool operator==(const Vector3 &p_v) const;
	inline bool operator!=(const Vector3 &p_v) const;
	inline bool operator<(const Vector3 &p_v) const;
	inline bool operator<=(const Vector3 &p_v) const;
	inline bool operator>(const Vector3 &p_v) const;
	inline bool operator>=(const Vector3 &p_v) const;

	operator String() const;
	operator Vector3i() const;

	inline Vector3() {}
	inline Vector3(real_t p_x, real_t p_y, real_t p_z) {
		x = p_x;
		y = p_y;
		z = p_z;
	}
};

Vector3 Vector3::cross(const Vector3 &p_b) const {
	Vector3 ret(
			(y * p_b.z) - (z * p_b.y),
			(z * p_b.x) - (x * p_b.z),
			(x * p_b.y) - (y * p_b.x));

	return ret;
}

real_t Vector3::dot(const Vector3 &p_b) const {
	return x * p_b.x + y * p_b.y + z * p_b.z;
}

Vector3 Vector3::abs() const {
	return Vector3(Math::abs(x), Math::abs(y), Math::abs(z));
}

Vector3 Vector3::sign() const {
	return Vector3(Math::sign(x), Math::sign(y), Math::sign(z));
}

Vector3 Vector3::floor() const {
	return Vector3(Math::floor(x), Math::floor(y), Math::floor(z));
}

Vector3 Vector3::ceil() const {
	return Vector3(Math::ceil(x), Math::ceil(y), Math::ceil(z));
}

Vector3 Vector3::round() const {
	return Vector3(Math::round(x), Math::round(y), Math::round(z));
}

Vector3 Vector3::lerp(const Vector3 &p_to, real_t p_weight) const {
	return Vector3(
			x + (p_weight * (p_to.x - x)),
			y + (p_weight * (p_to.y - y)),
			z + (p_weight * (p_to.z - z)));
}

Vector3 Vector3::slerp(const Vector3 &p_to, real_t p_weight) const {
	real_t theta = angle_to(p_to);
	return rotated(cross(p_to).normalized(), theta * p_weight);
}

real_t Vector3::distance_to(const Vector3 &p_to) const {
	return (p_to - *this).length();
}

real_t Vector3::distance_squared_to(const Vector3 &p_to) const {
	return (p_to - *this).length_squared();
}

Vector3 Vector3::posmod(const real_t p_mod) const {
	return Vector3(Math::fposmod(x, p_mod), Math::fposmod(y, p_mod), Math::fposmod(z, p_mod));
}

Vector3 Vector3::posmodv(const Vector3 &p_modv) const {
	return Vector3(Math::fposmod(x, p_modv.x), Math::fposmod(y, p_modv.y), Math::fposmod(z, p_modv.z));
}

Vector3 Vector3::project(const Vector3 &p_to) const {
	return p_to * (dot(p_to) / p_to.length_squared());
}

real_t Vector3::angle_to(const Vector3 &p_to) const {
	return Math::atan2(cross(p_to).length(), dot(p_to));
}

Vector3 Vector3::direction_to(const Vector3 &p_to) const {
	Vector3 ret(p_to.x - x, p_to.y - y, p_to.z - z);
	ret.normalize();
	return ret;
}

/* Operators */

Vector3 &Vector3::operator+=(const Vector3 &p_v) {
	x += p_v.x;
	y += p_v.y;
	z += p_v.z;
	return *this;
}

Vector3 Vector3::operator+(const Vector3 &p_v) const {
	return Vector3(x + p_v.x, y + p_v.y, z + p_v.z);
}

Vector3 &Vector3::operator-=(const Vector3 &p_v) {
	x -= p_v.x;
	y -= p_v.y;
	z -= p_v.z;
	return *this;
}

Vector3 Vector3::operator-(const Vector3 &p_v) const {
	return Vector3(x - p_v.x, y - p_v.y, z - p_v.z);
}

Vector3 &Vector3::operator*=(const Vector3 &p_v) {
	x *= p_v.x;
	y *= p_v.y;
	z *= p_v.z;
	return *this;
}

Vector3 Vector3::operator*(const Vector3 &p_v) const {
	return Vector3(x * p_v.x, y * p_v.y, z * p_v.z);
}

Vector3 &Vector3::operator/=(const Vector3 &p_v) {
	x /= p_v.x;
	y /= p_v.y;
	z /= p_v.z;
	return *this;
}

Vector3 Vector3::operator/(const Vector3 &p_v) const {
	return Vector3(x / p_v.x, y / p_v.y, z / p_v.z);
}

Vector3 &Vector3::operator*=(real_t p_scalar) {
	x *= p_scalar;
	y *= p_scalar;
	z *= p_scalar;
	return *this;
}

inline Vector3 operator*(real_t p_scalar, const Vector3 &p_vec) {
	return p_vec * p_scalar;
}

Vector3 Vector3::operator*(real_t p_scalar) const {
	return Vector3(x * p_scalar, y * p_scalar, z * p_scalar);
}

Vector3 &Vector3::operator/=(real_t p_scalar) {
	x /= p_scalar;
	y /= p_scalar;
	z /= p_scalar;
	return *this;
}

Vector3 Vector3::operator/(real_t p_scalar) const {
	return Vector3(x / p_scalar, y / p_scalar, z / p_scalar);
}

Vector3 Vector3::operator-() const {
	return Vector3(-x, -y, -z);
}

bool Vector3::operator==(const Vector3 &p_v) const {
	return x == p_v.x && y == p_v.y && z == p_v.z;
}

bool Vector3::operator!=(const Vector3 &p_v) const {
	return x != p_v.x || y != p_v.y || z != p_v.z;
}

bool Vector3::operator<(const Vector3 &p_v) const {
	if (x == p_v.x) {
		if (y == p_v.y) {
			return z < p_v.z;
		}
		return y < p_v.y;
	}
	return x < p_v.x;
}

bool Vector3::operator>(const Vector3 &p_v) const {
	if (x == p_v.x) {
		if (y == p_v.y) {
			return z > p_v.z;
		}
		return y > p_v.y;
	}
	return x > p_v.x;
}

bool Vector3::operator<=(const Vector3 &p_v) const {
	if (x == p_v.x) {
		if (y == p_v.y) {
			return z <= p_v.z;
		}
		return y < p_v.y;
	}
	return x < p_v.x;
}

bool Vector3::operator>=(const Vector3 &p_v) const {
	if (x == p_v.x) {
		if (y == p_v.y) {
			return z >= p_v.z;
		}
		return y > p_v.y;
	}
	return x > p_v.x;
}

inline Vector3 vec3_cross(const Vector3 &p_a, const Vector3 &p_b) {
	return p_a.cross(p_b);
}

inline real_t vec3_dot(const Vector3 &p_a, const Vector3 &p_b) {
	return p_a.dot(p_b);
}

real_t Vector3::length() const {
	real_t x2 = x * x;
	real_t y2 = y * y;
	real_t z2 = z * z;

	return Math::sqrt(x2 + y2 + z2);
}

real_t Vector3::length_squared() const {
	real_t x2 = x * x;
	real_t y2 = y * y;
	real_t z2 = z * z;

	return x2 + y2 + z2;
}

void Vector3::normalize() {
	real_t lengthsq = length_squared();
	if (lengthsq == (real_t)0.0) {
		x = y = z = (real_t)0.0;
	} else {
		real_t length = Math::sqrt(lengthsq);
		x /= length;
		y /= length;
		z /= length;
	}
}

Vector3 Vector3::normalized() const {
	Vector3 v = *this;
	v.normalize();
	return v;
}

bool Vector3::is_normalized() const {
	// use length_squared() instead of length() to avoid sqrt(), makes it more stringent.
	return Math::is_equal_approx(length_squared(), (real_t)1.0, (real_t)UNIT_EPSILON);
}

Vector3 Vector3::inverse() const {
	return Vector3((real_t)1.0 / x, (real_t)1.0 / y, (real_t)1.0 / z);
}

void Vector3::zero() {
	x = y = z = (real_t)0.0;
}

// slide returns the component of the vector along the given plane, specified by its normal vector.
Vector3 Vector3::slide(const Vector3 &p_normal) const {
#ifdef MATH_CHECKS
	ERR_FAIL_COND_V(!p_normal.is_normalized(), Vector3());
#endif
	return *this - p_normal * this->dot(p_normal);
}

Vector3 Vector3::bounce(const Vector3 &p_normal) const {
	return -reflect(p_normal);
}

Vector3 Vector3::reflect(const Vector3 &p_normal) const {
#ifdef MATH_CHECKS
	ERR_FAIL_COND_V(!p_normal.is_normalized(), Vector3());
#endif
	return 2.0 * p_normal * this->dot(p_normal) - *this;
}

} // namespace godot

#endif // GODOT_VECTOR3_HPP
