import { DataProvider } from "react-admin";
import axios from 'axios';

const apiUrl = 'http://localhost:3000/api/graphql';

export const dataProvider: DataProvider = {
    getList: (resource, params) => {
        const { page, perPage } = params.pagination;
        const { field, order } = params.sort;

        return axios.post(apiUrl, {
            query: `
            query {
              notes (
                orderBy: { ${field}: ${order} },
                pagination: { page: { limit: ${perPage}, page: ${page - 1} }}
              ) {
                nodes {
                  id
                  title
                  content
                }
                paginationInfo {
                  pages
                  current
                  offset
                  total
                }
              }
            }
            `
        })
            .then((response) => {
                const { nodes, paginationInfo } = response.data.data.notes;
                return {
                    data: nodes,
                    total: paginationInfo.total,
                };
            });
    },

    getOne: (resource, params) => {
        return axios.post(apiUrl, {
            query: `
            query {
              notes(filters: {id: {eq: ${params.id}}}) {
                nodes {
                  id
                  title
                  content
                }
              }
            }
            `
        })
            .then((response) => {
                const { nodes } = response.data.data.notes;
                return {
                    data: nodes[0],
                };
            });
    },

    getMany: (resource, params) => { },

    getManyReference: (resource, params) => { },

    update: (resource, params) => { },

    updateMany: (resource, params) => { },

    create: (resource, params) => { },

    delete: (resource, params) => { },

    deleteMany: (resource, params) => { },
};
