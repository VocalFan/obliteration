#pragma once

#include "core.h"

#include <stdlib.h>

template<typename T>
class Rust final {
public:
    Rust() : m_ptr(nullptr) {}
    explicit Rust(T *ptr) : m_ptr(ptr) {}
    Rust(const Rust &) = delete;

    Rust(Rust &&other) : m_ptr(other.m_ptr)
    {
        other.m_ptr = nullptr;
    }

    ~Rust()
    {
        free();
    }

    Rust &operator=(const Rust &) = delete;

    Rust &operator=(Rust &&other)
    {
        free();

        m_ptr = other.m_ptr;
        other.m_ptr = nullptr;

        return *this;
    }

    Rust &operator=(T *ptr)
    {
        free();
        m_ptr = ptr;
        return *this;
    }

    operator T *() { return m_ptr; }
    operator bool() const { return m_ptr != nullptr; }

    T **operator&()
    {
        free();
        return &m_ptr;
    }

    T *get() { return m_ptr; }
    void free();
private:
    T *m_ptr;
};

template<>
inline void Rust<char>::free()
{
    ::free(m_ptr);
    m_ptr = nullptr;
}

template<>
inline void Rust<Param>::free()
{
    if (m_ptr) {
        param_close(m_ptr);
        m_ptr = nullptr;
    }
}

template<>
inline void Rust<Pkg>::free()
{
    if (m_ptr) {
        pkg_close(m_ptr);
        m_ptr = nullptr;
    }
}

template<>
inline void Rust<RustError>::free()
{
    if (m_ptr) {
        error_free(m_ptr);
        m_ptr = nullptr;
    }
}

template<>
inline void Rust<Vmm>::free()
{
    if (m_ptr) {
        vmm_free(m_ptr);
        m_ptr = nullptr;
    }
}
