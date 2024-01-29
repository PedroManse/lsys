// +build 18

package verfitting

import (
	"sync/atomic"
)

type AUint64 struct {
	v uint64
}

func (AU *AUint64) Load() uint64 {
	return atomic.LoadUint64(&AU.v)
}

func (AU *AUint64) Store(newv uint64) {
	atomic.StoreUint64(&AU.v, newv)
}

func (AU *AUint64) Add(delta uint64) uint64 {
	return atomic.AddUint64(&AU.v, delta)
}

